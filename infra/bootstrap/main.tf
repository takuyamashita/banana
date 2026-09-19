# 環境(infra/envs/)を作る前に、アカウントに1回だけ作るもの。管理者が手元から apply する。
#
# - Terraform の state の置き場(環境ごと、この bootstrap 自身の分も)
# - Lambda の zip の置き場
# - GitHub Actions が OIDC で引き受けるロール(アクセスキーを GitHub に置かない)
#   - <環境>      : デプロイ(terraform apply まで)。GitHub Environment「<環境>」のジョブだけが引き受けられる
#   - <環境>-plan : plan だけ(読み取り)。GitHub Environment「<環境>-plan」のジョブだけが引き受けられる
#
# apply の後、出力のロール ARN を GitHub Environments の変数(AWS_DEPLOY_ROLE_ARN・AWS_PLAN_ROLE_ARN)に入れ、
# stg・prod の Environment には承認者(Required reviewers)を設定する。Environment の設定はコードにしていない

data "aws_caller_identity" "current" {}

locals {
  state_buckets = concat([for env in var.environments : "platform-terraform-state-${env}"], ["platform-terraform-state-bootstrap"])
}

# ---- state と成果物の置き場 ----

resource "aws_s3_bucket" "this" {
  for_each = toset(concat(local.state_buckets, [var.artifact_bucket]))
  bucket   = each.value
}

resource "aws_s3_bucket_public_access_block" "this" {
  for_each                = aws_s3_bucket.this
  bucket                  = each.value.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_server_side_encryption_configuration" "this" {
  for_each = aws_s3_bucket.this
  bucket   = each.value.id
  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

# state は壊したときに戻せるよう版を残す(成果物は git sha ごとに別のキーなので要らない)
resource "aws_s3_bucket_versioning" "this" {
  for_each = aws_s3_bucket.this
  bucket   = each.value.id
  versioning_configuration {
    status = contains(local.state_buckets, each.key) ? "Enabled" : "Suspended"
  }
}

data "aws_iam_policy_document" "tls_only" {
  for_each = aws_s3_bucket.this
  statement {
    effect    = "Deny"
    actions   = ["s3:*"]
    resources = [each.value.arn, "${each.value.arn}/*"]
    principals {
      type        = "*"
      identifiers = ["*"]
    }
    condition {
      test     = "Bool"
      variable = "aws:SecureTransport"
      values   = ["false"]
    }
  }
}

resource "aws_s3_bucket_policy" "tls_only" {
  for_each = aws_s3_bucket.this
  bucket   = each.value.id
  policy   = data.aws_iam_policy_document.tls_only[each.key].json
}

# Lambda は更新のときに zip を取り込むので、置いた zip は再デプロイ・ロールバックのためだけに残す
resource "aws_s3_bucket_lifecycle_configuration" "artifacts" {
  bucket = aws_s3_bucket.this[var.artifact_bucket].id
  rule {
    id     = "expire-old-artifacts"
    status = "Enabled"
    filter {}
    expiration {
      days = 180
    }
  }
}

# ---- GitHub Actions のロール ----

resource "aws_iam_openid_connect_provider" "github" {
  url            = "https://token.actions.githubusercontent.com"
  client_id_list = ["sts.amazonaws.com"]
}

# GitHub Environment ごとに引き受けられるロールを分ける。sub に Environment の名前が入る
data "aws_iam_policy_document" "assume" {
  for_each = toset(concat(var.environments, [for env in var.environments : "${env}-plan"]))
  statement {
    actions = ["sts:AssumeRoleWithWebIdentity"]
    principals {
      type        = "Federated"
      identifiers = [aws_iam_openid_connect_provider.github.arn]
    }
    condition {
      test     = "StringEquals"
      variable = "token.actions.githubusercontent.com:aud"
      values   = ["sts.amazonaws.com"]
    }
    condition {
      test     = "StringEquals"
      variable = "token.actions.githubusercontent.com:sub"
      values   = ["repo:${var.github_repository}:environment:${each.key}"]
    }
  }
}

# デプロイ: terraform apply で IAM ロールも作るので、広い権限が要る。
# 引き受けられるのは承認を経た Environment のジョブだけ、という形で絞る
resource "aws_iam_role" "deploy" {
  for_each             = toset(var.environments)
  name                 = "github-deploy-${each.key}"
  assume_role_policy   = data.aws_iam_policy_document.assume[each.key].json
  max_session_duration = 3600
}

resource "aws_iam_role_policy_attachment" "deploy" {
  for_each   = aws_iam_role.deploy
  role       = each.value.name
  policy_arn = "arn:aws:iam::aws:policy/AdministratorAccess"
}

# plan: 読むだけ。ただし state のロック(S3 の .tflock)を置けることと、
# Terraform が state を最新にするときにシークレットの値を読むこと(aws_secretsmanager_secret_version)は要る
resource "aws_iam_role" "plan" {
  for_each             = toset(var.environments)
  name                 = "github-plan-${each.key}"
  assume_role_policy   = data.aws_iam_policy_document.assume["${each.key}-plan"].json
  max_session_duration = 3600
}

resource "aws_iam_role_policy_attachment" "plan" {
  for_each   = aws_iam_role.plan
  role       = each.value.name
  policy_arn = "arn:aws:iam::aws:policy/ReadOnlyAccess"
}

data "aws_iam_policy_document" "plan" {
  for_each = toset(var.environments)
  statement {
    actions   = ["s3:PutObject", "s3:DeleteObject"]
    resources = ["${aws_s3_bucket.this["platform-terraform-state-${each.key}"].arn}/*.tflock"]
  }
  statement {
    actions   = ["secretsmanager:GetSecretValue"]
    resources = ["arn:aws:secretsmanager:*:${data.aws_caller_identity.current.account_id}:secret:platform/${each.key == "prod" ? "prd" : each.key}/*"]
  }
}

resource "aws_iam_role_policy" "plan" {
  for_each = aws_iam_role.plan
  name     = "terraform-plan"
  role     = each.value.name
  policy   = data.aws_iam_policy_document.plan[each.key].json
}
