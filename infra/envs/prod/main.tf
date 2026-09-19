# 本番。AZ の障害で止まらず、消せず、多要素認証を必須にする
module "stack" {
  source     = "../../modules/stack"
  name       = "platform-prod"
  env        = "prd" # APP_ENV(config/prd.toml)
  cidr_block = "10.0.0.0/16"

  image_tag                  = var.image_tag
  lambda_artifact_bucket     = var.lambda_artifact_bucket
  lambda_artifact_key        = var.lambda_artifact_key
  api_domain                 = var.api_domain
  web_domain                 = var.web_domain
  alb_certificate_arn        = var.alb_certificate_arn
  cloudfront_certificate_arn = var.cloudfront_certificate_arn
  adot_layer_arn             = var.adot_layer_arn
  alarm_email                = var.alarm_email

  deletion_protection             = true
  db_instance_class               = "db.m7g.large"
  db_multi_az                     = true
  db_performance_insights_enabled = true
  desired_count                   = 2
  max_count                       = 6
  nat_gateway_per_az              = true
  container_insights              = true
  mfa_configuration               = "ON"
}
