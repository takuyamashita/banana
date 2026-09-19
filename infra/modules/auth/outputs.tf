data "aws_region" "current" {}

output "user_pool_id" {
  value = aws_cognito_user_pool.this.id
}

output "user_pool_arn" {
  value = aws_cognito_user_pool.this.arn
}

output "web_client_id" {
  value = aws_cognito_user_pool_client.web.id
}

# config/{env}.toml の auth.issuer とフロントの config.json の oidc.authority に入れる値
output "issuer" {
  value = "https://cognito-idp.${data.aws_region.current.region}.amazonaws.com/${aws_cognito_user_pool.this.id}"
}

# マネージドログインの URL。フロントの config.json のログアウト先(/logout)と失効先(/oauth2/revoke)に使う。
# Cognito の OIDC ディスカバリには end_session_endpoint が載らないため
output "hosted_ui_url" {
  value = "https://${aws_cognito_user_pool_domain.this.domain}.auth.${data.aws_region.current.region}.amazoncognito.com"
}
