# RDS for MySQL 8.4(compose.yaml の mysql:8.4 と揃える)。
#
# ユーザーは2つに分ける。管理者は RDS がパスワードを作って Secrets Manager で管理し(ローテーションも RDS)、
# migrate だけが使う。アプリ(server・Lambda)は読み書きだけのユーザーで接続し、テーブルを変えられない。
# アプリ用のユーザーは migrate がマイグレーションの後に作る(Terraform からは VPC の中の DB に届かないため)
resource "aws_db_subnet_group" "this" {
  name       = var.name
  subnet_ids = var.subnet_ids
}

resource "aws_security_group" "this" {
  name        = "${var.name}-db"
  description = "MySQL for ${var.name}"
  vpc_id      = var.vpc_id
}

# SG の ID は apply まで決まらないので for_each(toset)ではなく count を使う
resource "aws_vpc_security_group_ingress_rule" "mysql" {
  count                        = length(var.allowed_security_group_ids)
  security_group_id            = aws_security_group.this.id
  referenced_security_group_id = var.allowed_security_group_ids[count.index]
  ip_protocol                  = "tcp"
  from_port                    = 3306
  to_port                      = 3306
  description                  = "MySQL from application"
}

resource "aws_db_parameter_group" "this" {
  name   = var.name
  family = "mysql8.4"

  # 時刻は UTC で保存する。分離レベルはアプリが接続ごとに READ COMMITTED にするが、既定も揃えておく
  parameter {
    name  = "time_zone"
    value = "UTC"
  }
  parameter {
    name  = "transaction_isolation"
    value = "READ-COMMITTED"
  }
  parameter {
    name  = "require_secure_transport"
    value = "1"
  }
}

# migrate が作るアプリ用のユーザーのパスワード。記号は使わない(migrate が SQL の文に埋め込むため)
resource "random_password" "app" {
  length  = 32
  special = false
}

resource "aws_db_instance" "this" {
  identifier                   = var.name
  engine                       = "mysql"
  engine_version               = "8.4"
  instance_class               = var.instance_class
  allocated_storage            = 20
  max_allocated_storage        = 100
  storage_encrypted            = true
  db_name                      = "platform"
  username                     = "admin"
  manage_master_user_password  = true
  db_subnet_group_name         = aws_db_subnet_group.this.name
  vpc_security_group_ids       = [aws_security_group.this.id]
  parameter_group_name         = aws_db_parameter_group.this.name
  multi_az                     = var.multi_az
  backup_retention_period      = 7
  deletion_protection          = var.deletion_protection
  skip_final_snapshot          = !var.deletion_protection
  final_snapshot_identifier    = var.deletion_protection ? "${var.name}-final" : null
  performance_insights_enabled = var.performance_insights_enabled
  auto_minor_version_upgrade   = true
  copy_tags_to_snapshot        = true
}

# アプリ用のユーザー。migrate が読んで、この名前とパスワードでユーザーを作る。
# パスワードは tfstate にも残る。state の S3 バケットは暗号化・アクセス制限を前提にする
resource "aws_secretsmanager_secret" "app_user" {
  name        = "platform/${var.env}/database-app-user"
  description = "MySQL user for the application (created by migrate)"
}

resource "aws_secretsmanager_secret_version" "app_user" {
  secret_id     = aws_secretsmanager_secret.app_user.id
  secret_string = jsonencode({ username = "app", password = random_password.app.result })
}

# アプリが接続に使う文字列(アプリ用のユーザー)
resource "aws_secretsmanager_secret" "database_url" {
  name        = "platform/${var.env}/database-url"
  description = "MySQL connection string for the application"
}

resource "aws_secretsmanager_secret_version" "database_url" {
  secret_id     = aws_secretsmanager_secret.database_url.id
  secret_string = "mysql://app:${random_password.app.result}@${aws_db_instance.this.address}:${aws_db_instance.this.port}/${aws_db_instance.this.db_name}?ssl-mode=required"
}
