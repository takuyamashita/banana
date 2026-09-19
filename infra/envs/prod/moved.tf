# 構成を modules/stack にまとめる前に作った環境の state を、新しい場所に付け替える
# (付け替えないと、DB などを消して作り直す plan になる)。全環境で apply したら消してよい
moved {
  from = module.network
  to   = module.stack.module.network
}

moved {
  from = module.auth
  to   = module.stack.module.auth
}

moved {
  from = module.messaging
  to   = module.stack.module.messaging
}

moved {
  from = module.database
  to   = module.stack.module.database
}

moved {
  from = module.backend
  to   = module.stack.module.backend
}

moved {
  from = module.payout_dispatcher
  to   = module.stack.module.payout_dispatcher
}

moved {
  from = module.frontend
  to   = module.stack.module.frontend
}

moved {
  from = aws_secretsmanager_secret.payout_api_key
  to   = module.stack.aws_secretsmanager_secret.payout_api_key
}
