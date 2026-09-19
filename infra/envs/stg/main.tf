locals {
  name = "platform-stg"
  env  = "stg" # APP_ENV(config/stg.toml)
}

module "network" {
  source     = "../../modules/network"
  name       = local.name
  cidr_block = "10.0.0.0/16"
}

module "auth" {
  source              = "../../modules/auth"
  name                = local.name
  domain_prefix       = local.name
  callback_urls       = ["https://${var.web_domain}/"]
  deletion_protection = false
}

module "messaging" {
  source = "../../modules/messaging"
  name   = "payroll-events-stg"
}

module "database" {
  source                     = "../../modules/database"
  name                       = local.name
  env                        = local.env
  vpc_id                     = module.network.vpc_id
  subnet_ids                 = module.network.private_subnet_ids
  allowed_security_group_ids = [module.backend.task_security_group_id, module.payout_dispatcher.security_group_id]
  instance_class             = "db.t4g.small"
  multi_az                   = false
  deletion_protection        = false
}

module "backend" {
  source                  = "../../modules/backend-service"
  name                    = local.name
  env                     = local.env
  vpc_id                  = module.network.vpc_id
  vpc_cidr                = module.network.vpc_cidr
  public_subnet_ids       = module.network.public_subnet_ids
  private_subnet_ids      = module.network.private_subnet_ids
  image                   = "${module.backend.ecr_repository_url}:${var.image_tag}"
  certificate_arn         = var.alb_certificate_arn
  database_url_secret_arn = module.database.database_url_secret_arn
  queue_arn               = module.messaging.queue_arn
  user_pool_arn           = module.auth.user_pool_arn
  app_environment         = local.server_environment
  desired_count           = 2
  deletion_protection     = false
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
  name        = "platform/${local.env}/payout-api-key"
  description = "API key for the bank payout API"
}

data "aws_iam_policy_document" "payout_dispatcher" {
  statement {
    actions   = ["secretsmanager:GetSecretValue"]
    resources = [module.database.database_url_secret_arn, aws_secretsmanager_secret.payout_api_key.arn]
  }
}

module "payout_dispatcher" {
  source          = "../../modules/lambda-function"
  name            = "${local.name}-payout-dispatcher"
  env             = local.env
  artifact_bucket = var.lambda_artifact_bucket
  artifact_key    = var.lambda_artifact_key
  vpc_id          = module.network.vpc_id
  vpc_cidr        = module.network.vpc_cidr
  subnet_ids      = module.network.private_subnet_ids
  sqs_queue_arn   = module.messaging.queue_arn
  policy_json     = data.aws_iam_policy_document.payout_dispatcher.json
  adot_layer_arn  = var.adot_layer_arn
  environment = {
    APP__SECRETS__DATABASE_URL_SECRET_ID   = module.database.database_url_secret_arn
    APP__SECRETS__PAYOUT_API_KEY_SECRET_ID = aws_secretsmanager_secret.payout_api_key.arn
    # 同時実行数(maximum_concurrency)× この接続数が RDS の上限に収まるようにする
    APP__DATABASE__MAX_CONNECTIONS = "2"
  }
}

module "frontend" {
  source          = "../../modules/frontend-hosting"
  bucket_name     = "${local.name}-web"
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
