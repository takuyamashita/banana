# サービスを2つ(給与・勤怠)にしたときの付け替え。付け替えないと、作り直す plan になる。全環境で apply したら消してよい。
# (リソースの名前が変わったもの(ECR・シークレットなど)は、付け替えても作り直しになる)

# 給与のサーバーは backend-service の1つ目になった
moved {
  from = module.backend
  to   = module.payroll
}

# ALB とクラスターは、サービスで共有する load-balancer に移した
moved {
  from = module.payroll.aws_lb.this
  to   = module.load_balancer.aws_lb.this
}

moved {
  from = module.payroll.aws_lb_listener.https
  to   = module.load_balancer.aws_lb_listener.https
}

moved {
  from = module.payroll.aws_security_group.alb
  to   = module.load_balancer.aws_security_group.alb
}

moved {
  from = module.payroll.aws_vpc_security_group_ingress_rule.alb_https
  to   = module.load_balancer.aws_vpc_security_group_ingress_rule.alb_https
}

moved {
  from = module.payroll.aws_ecs_cluster.this
  to   = module.load_balancer.aws_ecs_cluster.this
}

# 給与の出来事のキューは、振込の受け手のキューになった(給与の出来事は SNS のトピックから配る)
moved {
  from = module.messaging
  to   = module.payout_queue
}

# DB のアプリ用ユーザー・接続文字列はサービスごとになった
moved {
  from = module.database.random_password.app
  to   = module.database.random_password.app["payroll"]
}

moved {
  from = module.database.aws_secretsmanager_secret.app_user
  to   = module.database.aws_secretsmanager_secret.app_user["payroll"]
}

moved {
  from = module.database.aws_secretsmanager_secret_version.app_user
  to   = module.database.aws_secretsmanager_secret_version.app_user["payroll"]
}

moved {
  from = module.database.aws_secretsmanager_secret.database_url
  to   = module.database.aws_secretsmanager_secret.database_url["payroll"]
}

moved {
  from = module.database.aws_secretsmanager_secret_version.database_url
  to   = module.database.aws_secretsmanager_secret_version.database_url["payroll"]
}
