# RDS for MySQL 8.4(compose.yaml の mysql:8.4 と揃える)。
# アプリは接続文字列を Secrets Manager から起動時に読む(bootstrap の secrets.database_url_secret_id)
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

resource "random_password" "master" {
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
  username                     = "platform"
  password                     = random_password.master.result
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

# パスワードは tfstate にも残る。state の S3 バケットは暗号化・アクセス制限を前提にする
resource "aws_secretsmanager_secret" "database_url" {
  name        = "platform/${var.env}/database-url"
  description = "MySQL connection string for the application"
}

resource "aws_secretsmanager_secret_version" "database_url" {
  secret_id     = aws_secretsmanager_secret.database_url.id
  secret_string = "mysql://${aws_db_instance.this.username}:${random_password.master.result}@${aws_db_instance.this.address}:${aws_db_instance.this.port}/${aws_db_instance.this.db_name}?ssl-mode=required"
}
