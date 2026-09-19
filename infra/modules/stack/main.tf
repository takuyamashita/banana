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

module "messaging" {
  source        = "../messaging"
  name          = "payroll-events-${var.env}"
  alarm_actions = [aws_sns_topic.alarms.arn]
}

module "database" {
  source                       = "../database"
  name                         = var.name
  env                          = var.env
  vpc_id                       = module.network.vpc_id
  subnet_ids                   = module.network.private_subnet_ids
  allowed_security_group_ids   = [module.backend.task_security_group_id, module.payout_dispatcher.security_group_id]
  instance_class               = var.db_instance_class
  performance_insights_enabled = var.db_performance_insights_enabled
  multi_az                     = var.db_multi_az
  deletion_protection          = var.deletion_protection
}

module "backend" {
  source                       = "../backend-service"
  name                         = var.name
  env                          = var.env
  vpc_id                       = module.network.vpc_id
  public_subnet_ids            = module.network.public_subnet_ids
  private_subnet_ids           = module.network.private_subnet_ids
  image                        = "${module.backend.ecr_repository_url}:${var.image_tag}"
  certificate_arn              = var.alb_certificate_arn
  database_security_group_id   = module.database.security_group_id
  database_url_secret_arn      = module.database.database_url_secret_arn
  database_admin_secret_arn    = module.database.admin_secret_arn
  database_app_user_secret_arn = module.database.app_user_secret_arn
  queue_arn                    = module.messaging.queue_arn
  user_pool_arn                = module.auth.user_pool_arn
  app_environment              = local.server_environment
  desired_count                = var.desired_count
  max_count                    = var.max_count
  container_insights           = var.container_insights
  deletion_protection          = var.deletion_protection
  alarm_actions                = [aws_sns_topic.alarms.arn]
}

# インフラが決める値は config/{env}.toml に書かず、ここから環境変数(APP__SECTION__KEY)で渡す
locals {
  server_environment = {
    APP__AUTH__ISSUER                    = module.auth.issuer
    APP__AUTH__COGNITO_CLIENT_ID         = module.auth.web_client_id
    APP__AUTH__COGNITO_USER_POOL_ID      = module.auth.user_pool_id
    APP__MESSAGING__QUEUE_URL            = module.messaging.queue_url
    APP__SECRETS__DATABASE_URL_SECRET_ID = module.database.database_url_secret_arn
    APP__SERVER__CORS_ALLOWED_ORIGINS    = "https://${var.web_domain}"
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
    resources = [module.database.database_url_secret_arn, aws_secretsmanager_secret.payout_api_key.arn]
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
  sqs_queue_arn              = module.messaging.queue_arn
  policy_json                = data.aws_iam_policy_document.payout_dispatcher.json
  adot_layer_arn             = var.adot_layer_arn
  alarm_actions              = [aws_sns_topic.alarms.arn]
  environment = {
    APP__SECRETS__DATABASE_URL_SECRET_ID   = module.database.database_url_secret_arn
    APP__SECRETS__PAYOUT_API_KEY_SECRET_ID = aws_secretsmanager_secret.payout_api_key.arn
    # 同時実行数(maximum_concurrency)× この接続数が RDS の上限に収まるようにする
    APP__DATABASE__MAX_CONNECTIONS = "2"
  }
}

module "frontend" {
  source          = "../frontend-hosting"
  bucket_name     = "${var.name}-web"
  aliases         = [var.web_domain]
  certificate_arn = var.cloudfront_certificate_arn
  runtime_config = {
    apiBaseUrl = "https://${var.api_domain}"
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
