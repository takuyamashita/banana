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
  desired_count           = 2
  deletion_protection     = false
}

data "aws_iam_policy_document" "payout_dispatcher" {
  statement {
    actions   = ["secretsmanager:GetSecretValue"]
    resources = [module.database.database_url_secret_arn]
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
