# 開発用。止まってもよいので安く、消しやすくする
module "stack" {
  source     = "../../modules/stack"
  name       = "platform-dev"
  env        = "dev" # APP_ENV(config/dev.toml)
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

  deletion_protection             = false
  db_instance_class               = "db.t4g.micro"
  db_multi_az                     = false
  db_performance_insights_enabled = false
  desired_count                   = 1
  max_count                       = 2
  nat_gateway_per_az              = false
  container_insights              = false
  mfa_configuration               = "OPTIONAL"
}
