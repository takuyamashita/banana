# 1つの環境(dev・stg・prod)をまるごと組み立てる。環境ごとの差は、この入力(規模と保護の強さ)だけにする。
# envs/<環境>/ は、ここに値を渡すだけ

module "network" {
  source             = "../network"
  name               = var.name
  cidr_block         = var.cidr_block
  nat_gateway_per_az = var.nat_gateway_per_az
}

module "auth" {
  source              = "../auth"
  name                = var.name
  domain_prefix       = var.name
  callback_urls       = ["https://${var.web_domain}/"]
  mfa_configuration   = var.mfa_configuration
  deletion_protection = var.deletion_protection
}

# アラームの通知先。受け取るメールアドレスを指定したときだけ購読を作る(購読は届いたメールで承認する)
resource "aws_sns_topic" "alarms" {
  name = "${var.name}-alarms"
}

resource "aws_sns_topic_subscription" "alarm_email" {
  count     = var.alarm_email == null ? 0 : 1
  topic_arn = aws_sns_topic.alarms.arn
  protocol  = "email"
  endpoint  = var.alarm_email
}

# ---- サービスの間の出来事 ----
# 出す側のサービスごとにトピックを1つ、受け手ごとにキューを1つ置く。キューは要る種類(event_type)だけを購読する
module "payroll_events" {
  source = "../event-topic"
  name   = "${var.name}-payroll-events"
}

module "timesheet_events" {
  source = "../event-topic"
  name   = "${var.name}-timesheet-events"
}

# 給与明細の確定 → 振込(payout-dispatcher Lambda)
module "payout_queue" {
  source        = "../event-queue"
  name          = "${var.name}-payroll-payout"
  alarm_actions = [aws_sns_topic.alarms.arn]
  subscriptions = [{ topic_arn = module.payroll_events.arn, event_types = ["payslip.finalized"] }]
}

# 派遣社員・案件の登録 → 勤怠(timesheet server の受け手)
module "timesheet_inbox" {
  source                     = "../event-queue"
  name                       = "${var.name}-timesheet-inbox"
  visibility_timeout_seconds = 60
  alarm_actions              = [aws_sns_topic.alarms.arn]
  subscriptions              = [{ topic_arn = module.payroll_events.arn, event_types = ["staff.registered", "project.created"] }]
}

# 勤務表の承認 → 給与(payroll server の受け手)
module "payroll_inbox" {
  source                     = "../event-queue"
  name                       = "${var.name}-payroll-inbox"
  visibility_timeout_seconds = 60
  alarm_actions              = [aws_sns_topic.alarms.arn]
  subscriptions              = [{ topic_arn = module.timesheet_events.arn, event_types = ["timesheet.approved"] }]
}

module "database" {
  source   = "../database"
  name     = var.name
  env      = var.env
  services = ["payroll", "timesheet"]
  vpc_id   = module.network.vpc_id
  allowed_security_group_ids = [
    module.payroll.task_security_group_id,
    module.timesheet.task_security_group_id,
    module.payout_dispatcher.security_group_id,
  ]
  subnet_ids                   = module.network.private_subnet_ids
  instance_class               = var.db_instance_class
  performance_insights_enabled = var.db_performance_insights_enabled
  multi_az                     = var.db_multi_az
  deletion_protection          = var.deletion_protection
}

# ---- サービス ----
# ALB とクラスターは共有する。ブラウザからは同じ API のドメインで、RPC のパスでサービスに届く
module "load_balancer" {
  source              = "../load-balancer"
  name                = var.name
  vpc_id              = module.network.vpc_id
  public_subnet_ids   = module.network.public_subnet_ids
  certificate_arn     = var.alb_certificate_arn
  container_insights  = var.container_insights
  deletion_protection = var.deletion_protection
  alarm_actions       = [aws_sns_topic.alarms.arn]
}

# どのサービスにも同じ形で渡すもの
locals {
  service_common = {
    env                     = var.env
    vpc_id                  = module.network.vpc_id
    private_subnet_ids      = module.network.private_subnet_ids
    cluster_id              = module.load_balancer.cluster_id
    cluster_name            = module.load_balancer.cluster_name
    listener_arn            = module.load_balancer.listener_arn
    alb_security_group_id   = module.load_balancer.alb_security_group_id
    alb_arn_suffix          = module.load_balancer.alb_arn_suffix
    database_admin_secret   = module.database.admin_secret_arn
    database_security_group = module.database.security_group_id
  }
}

# 給与(payroll)。出来事を payroll_events に出し、payroll_inbox で勤怠の出来事を受ける。派遣社員のログインを Cognito に作る
data "aws_iam_policy_document" "payroll_task" {
  statement {
    actions   = ["sns:Publish"]
    resources = [module.payroll_events.arn]
  }
  statement {
    actions   = ["sqs:ReceiveMessage", "sqs:DeleteMessage", "sqs:GetQueueAttributes", "sqs:ChangeMessageVisibility"]
    resources = [module.payroll_inbox.queue_arn]
  }
  statement {
    actions   = ["cognito-idp:AdminCreateUser", "cognito-idp:AdminAddUserToGroup", "cognito-idp:AdminDeleteUser"]
    resources = [module.auth.user_pool_arn]
  }
}

module "payroll" {
  source                       = "../backend-service"
  name                         = "${var.name}-payroll"
  env_prefix                   = "PAYROLL"
  env                          = local.service_common.env
  vpc_id                       = local.service_common.vpc_id
  private_subnet_ids           = local.service_common.private_subnet_ids
  cluster_id                   = local.service_common.cluster_id
  cluster_name                 = local.service_common.cluster_name
  listener_arn                 = local.service_common.listener_arn
  listener_rule_priority       = 100
  path_patterns                = ["/acme.payroll.v1.*"]
  alb_security_group_id        = local.service_common.alb_security_group_id
  alb_arn_suffix               = local.service_common.alb_arn_suffix
  image                        = "${module.payroll.ecr_repository_url}:${var.image_tag}"
  database_security_group_id   = local.service_common.database_security_group
  database_url_secret_arn      = module.database.database_url_secret_arns["payroll"]
  database_admin_secret_arn    = local.service_common.database_admin_secret
  database_app_user_secret_arn = module.database.app_user_secret_arns["payroll"]
  task_policy_json             = data.aws_iam_policy_document.payroll_task.json
  app_environment = merge(local.auth_environment["PAYROLL"], {
    PAYROLL__USER_DIRECTORY__COGNITO_USER_POOL_ID = module.auth.user_pool_id
    PAYROLL__MESSAGING__TOPIC_ARN                 = module.payroll_events.arn
    PAYROLL__MESSAGING__INBOX_QUEUE_URL           = module.payroll_inbox.queue_url
    PAYROLL__SECRETS__DATABASE_URL_SECRET_ID      = module.database.database_url_secret_arns["payroll"]
  })
  desired_count = var.desired_count
  max_count     = var.max_count
  alarm_actions = [aws_sns_topic.alarms.arn]
}

# 勤怠(timesheet)。出来事を timesheet_events に出し、timesheet_inbox で給与の出来事を受ける
data "aws_iam_policy_document" "timesheet_task" {
  statement {
    actions   = ["sns:Publish"]
    resources = [module.timesheet_events.arn]
  }
  statement {
    actions   = ["sqs:ReceiveMessage", "sqs:DeleteMessage", "sqs:GetQueueAttributes", "sqs:ChangeMessageVisibility"]
    resources = [module.timesheet_inbox.queue_arn]
  }
}

module "timesheet" {
  source                       = "../backend-service"
  name                         = "${var.name}-timesheet"
  env_prefix                   = "TIMESHEET"
  env                          = local.service_common.env
  vpc_id                       = local.service_common.vpc_id
  private_subnet_ids           = local.service_common.private_subnet_ids
  cluster_id                   = local.service_common.cluster_id
  cluster_name                 = local.service_common.cluster_name
  listener_arn                 = local.service_common.listener_arn
  listener_rule_priority       = 200
  path_patterns                = ["/acme.timesheet.v1.*"]
  alb_security_group_id        = local.service_common.alb_security_group_id
  alb_arn_suffix               = local.service_common.alb_arn_suffix
  image                        = "${module.timesheet.ecr_repository_url}:${var.image_tag}"
  database_security_group_id   = local.service_common.database_security_group
  database_url_secret_arn      = module.database.database_url_secret_arns["timesheet"]
  database_admin_secret_arn    = local.service_common.database_admin_secret
  database_app_user_secret_arn = module.database.app_user_secret_arns["timesheet"]
  task_policy_json             = data.aws_iam_policy_document.timesheet_task.json
  app_environment = merge(local.auth_environment["TIMESHEET"], {
    TIMESHEET__MESSAGING__TOPIC_ARN            = module.timesheet_events.arn
    TIMESHEET__MESSAGING__INBOX_QUEUE_URL      = module.timesheet_inbox.queue_url
    TIMESHEET__SECRETS__DATABASE_URL_SECRET_ID = module.database.database_url_secret_arns["timesheet"]
  })
  desired_count = var.desired_count
  max_count     = var.max_count
  alarm_actions = [aws_sns_topic.alarms.arn]
}

# インフラが決める値は config/<サービス>/{env}.toml に書かず、ここから環境変数(<サービス>__SECTION__KEY)で渡す。
# トークンの検証と CORS はどのサービスも同じ
locals {
  auth_environment = {
    for prefix in ["PAYROLL", "TIMESHEET"] : prefix => {
      "${prefix}__AUTH__ISSUER"                 = module.auth.issuer
      "${prefix}__AUTH__COGNITO_CLIENT_ID"      = module.auth.web_client_id
      "${prefix}__SERVER__CORS_ALLOWED_ORIGINS" = "https://${var.web_domain}"
    }
  }
}

# 振込 API のキー。値は Terraform では持たず、作成後にコンソールか CLI で入れる(put-secret-value)
resource "aws_secretsmanager_secret" "payout_api_key" {
  name        = "platform/${var.env}/payout-api-key"
  description = "API key for the bank payout API"
}

data "aws_iam_policy_document" "payout_dispatcher" {
  statement {
    actions   = ["secretsmanager:GetSecretValue"]
    resources = [module.database.database_url_secret_arns["payroll"], aws_secretsmanager_secret.payout_api_key.arn]
  }
}

module "payout_dispatcher" {
  source                     = "../lambda-function"
  name                       = "${var.name}-payout-dispatcher"
  env                        = var.env
  artifact_bucket            = var.lambda_artifact_bucket
  artifact_key               = var.lambda_artifact_key
  vpc_id                     = module.network.vpc_id
  subnet_ids                 = module.network.private_subnet_ids
  database_security_group_id = module.database.security_group_id
  sqs_queue_arn              = module.payout_queue.queue_arn
  policy_json                = data.aws_iam_policy_document.payout_dispatcher.json
  adot_layer_arn             = var.adot_layer_arn
  alarm_actions              = [aws_sns_topic.alarms.arn]
  # 給与サービスの設定(config/payroll/)で動く
  environment = {
    PAYROLL__SECRETS__DATABASE_URL_SECRET_ID = module.database.database_url_secret_arns["payroll"]
    PAYROLL__PAYOUT__API_KEY_SECRET_ID       = aws_secretsmanager_secret.payout_api_key.arn
    # 同時実行数(maximum_concurrency)× この接続数が RDS の上限に収まるようにする
    PAYROLL__DATABASE__MAX_CONNECTIONS = "2"
  }
}

module "frontend" {
  source          = "../frontend-hosting"
  bucket_name     = "${var.name}-web"
  aliases         = [var.web_domain]
  certificate_arn = var.cloudfront_certificate_arn
  runtime_config = {
    # 給与と勤怠は同じ ALB(RPC のパスで振り分ける)なので、同じ URL
    apiBaseUrl          = "https://${var.api_domain}"
    timesheetApiBaseUrl = "https://${var.api_domain}"
    oidc = {
      authority = module.auth.issuer
      clientId  = module.auth.web_client_id
      # Cognito のログアウトは /logout に client_id と logout_uri(logout_urls のどれか)を渡す
      endSessionEndpoint      = "${module.auth.hosted_ui_url}/logout"
      revocationEndpoint      = "${module.auth.hosted_ui_url}/oauth2/revoke"
      postLogoutRedirectParam = "logout_uri"
    }
  }
}
