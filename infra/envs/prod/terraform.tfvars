# prod 環境の値。秘密情報は書かない(DB のパスワードは Terraform が生成し、振込 API のキーは Secrets Manager に手で入れる)。
# デプロイのたびに変わる値(image_tag・lambda_artifact_key)は deploy ワークフローが -var で渡す。
# example.com と REPLACE_ME は、実際のドメインと ACM 証明書・成果物のバケット(infra の外で先に作る)に置き換える
api_domain                 = "api.example.com"
web_domain                 = "app.example.com"
alb_certificate_arn        = "arn:aws:acm:ap-northeast-1:000000000000:certificate/REPLACE_ME"
cloudfront_certificate_arn = "arn:aws:acm:us-east-1:000000000000:certificate/REPLACE_ME"
lambda_artifact_bucket     = "platform-artifacts-REPLACE_ME"
