# Cognito。ローカルの Keycloak(compose/keycloak-realm.json)と同じ構成にそろえる:
# メールアドレスでログイン、ロールはグループ(admin / staff)、Web は PKCE の公開クライアント
resource "aws_cognito_user_pool" "this" {
  name = var.name

  # email をユーザー名として使う。このとき AdminCreateUser の Username は sub(UUID)になる
  username_attributes      = ["email"]
  auto_verified_attributes = ["email"]
  deletion_protection      = var.deletion_protection ? "ACTIVE" : "INACTIVE"

  password_policy {
    minimum_length                   = 12
    require_lowercase                = true
    require_uppercase                = true
    require_numbers                  = true
    require_symbols                  = false
    temporary_password_validity_days = 7
  }

  admin_create_user_config {
    # 利用者の作成は管理者(UserDirectory)だけが行う
    allow_admin_create_user_only = true
  }

  # 給与の情報を扱うので、認証アプリ(TOTP)での多要素認証を使えるようにする。
  # prod は必須(ON)。Cognito ではグループ(管理者だけ)ごとに必須にはできない
  mfa_configuration = var.mfa_configuration
  software_token_mfa_configuration {
    enabled = true
  }

  account_recovery_setting {
    recovery_mechanism {
      name     = "verified_email"
      priority = 1
    }
  }
}

resource "aws_cognito_user_group" "admin" {
  name         = "admin"
  user_pool_id = aws_cognito_user_pool.this.id
  description  = "給与の確定など管理操作ができる"
}

resource "aws_cognito_user_group" "staff" {
  name         = "staff"
  user_pool_id = aws_cognito_user_pool.this.id
  description  = "派遣社員。自分の給与明細だけ見られる"
}

resource "aws_cognito_user_pool_domain" "this" {
  domain       = var.domain_prefix
  user_pool_id = aws_cognito_user_pool.this.id
}

resource "aws_cognito_user_pool_client" "web" {
  name                                 = "${var.name}-web"
  user_pool_id                         = aws_cognito_user_pool.this.id
  generate_secret                      = false
  allowed_oauth_flows_user_pool_client = true
  allowed_oauth_flows                  = ["code"]
  allowed_oauth_scopes                 = ["openid", "email", "profile"]
  callback_urls                        = var.callback_urls
  logout_urls                          = var.callback_urls
  supported_identity_providers         = ["COGNITO"]
  explicit_auth_flows                  = ["ALLOW_REFRESH_TOKEN_AUTH", "ALLOW_USER_SRP_AUTH"]
  prevent_user_existence_errors        = "ENABLED"
  # ログアウトでリフレッシュトークンを失効させる(/oauth2/revoke)
  enable_token_revocation = true
  access_token_validity   = 15
  id_token_validity       = 15
  refresh_token_validity  = 30
  token_validity_units {
    access_token  = "minutes"
    id_token      = "minutes"
    refresh_token = "days"
  }
}
