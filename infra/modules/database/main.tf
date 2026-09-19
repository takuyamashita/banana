# RDS for MySQL 8.4(compose.yaml の mysql:8.4 と揃える)。1つの RDS に、サービスごとのデータベースを置く。
#
# 管理者は RDS がパスワードを作って Secrets Manager で管理し(ローテーションも RDS)、migrate だけが使う。
# アプリ(server・Lambda)は、サービスごとの読み書きだけのユーザーで接続する。テーブルを変えられず、
# 他のサービスのデータベースも読めない。データベースとアプリ用のユーザーは、各サービスの migrate が作る
# (Terraform からは VPC の中の DB に届かないため)
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

# migrate が作るアプリ用のユーザーのパスワード(サービスごと)。記号は使わない(migrate が SQL の文に埋め込むため)
resource "random_password" "app" {
  for_each = toset(var.services)
  length   = 32
  special  = false
}

resource "aws_db_instance" "this" {
  identifier            = var.name
  engine                = "mysql"
  engine_version        = "8.4"
  instance_class        = var.instance_class
  allocated_storage     = 20
  max_allocated_storage = 100
  storage_encrypted     = true
  # データベースは作らない(サービスごとのデータベースは、各サービスの migrate が作る)
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

# アプリ用のユーザー(サービスごと)。migrate が読んで、この名前とパスワードでユーザーを作る。
# パスワードは tfstate にも残る。state の S3 バケットは暗号化・アクセス制限を前提にする
resource "aws_secretsmanager_secret" "app_user" {
  for_each    = toset(var.services)
  name        = "platform/${var.env}/${each.key}/database-app-user"
  description = "MySQL user for the ${each.key} service (created by its migrate)"
}

resource "aws_secretsmanager_secret_version" "app_user" {
  for_each      = toset(var.services)
  secret_id     = aws_secretsmanager_secret.app_user[each.key].id
  secret_string = jsonencode({ username = each.key, password = random_password.app[each.key].result })
}

# サービスが接続に使う文字列(そのサービスのユーザーとデータベース)
resource "aws_secretsmanager_secret" "database_url" {
  for_each    = toset(var.services)
  name        = "platform/${var.env}/${each.key}/database-url"
  description = "MySQL connection string for the ${each.key} service"
}

resource "aws_secretsmanager_secret_version" "database_url" {
  for_each      = toset(var.services)
  secret_id     = aws_secretsmanager_secret.database_url[each.key].id
  secret_string = "mysql://${each.key}:${random_password.app[each.key].result}@${aws_db_instance.this.address}:${aws_db_instance.this.port}/${each.key}?ssl-mode=required"
}
