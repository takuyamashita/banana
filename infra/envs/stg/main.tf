# リリース前の確かめ用。タスクは prod と同じく2つにして、入れ替えの振る舞いを確かめる
module "stack" {
  source     = "../../modules/stack"
  name       = "platform-stg"
  env        = "stg" # APP_ENV(config/stg.toml)
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
  db_instance_class               = "db.t4g.small"
  db_multi_az                     = false
  db_performance_insights_enabled = false
  desired_count                   = 2
  max_count                       = 4
  nat_gateway_per_az              = false
  container_insights              = false
  mfa_configuration               = "OPTIONAL"
}
