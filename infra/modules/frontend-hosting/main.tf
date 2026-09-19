# SPA を S3(非公開)+ CloudFront(OAC)で配信する。
# ビルド成果物は全環境で同一にし、環境ごとの差は config.json だけにする(ここで生成して置く)。
#
# キャッシュの長さは置くときの Cache-Control で決める(デプロイのワークフロー)。
# ファイル名にハッシュが入る assets/ は1年、index.html は毎回確かめる
resource "aws_s3_bucket" "this" {
  bucket = var.bucket_name
}

resource "aws_s3_bucket_public_access_block" "this" {
  bucket                  = aws_s3_bucket.this.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_server_side_encryption_configuration" "this" {
  bucket = aws_s3_bucket.this.id
  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

resource "aws_s3_bucket_versioning" "this" {
  bucket = aws_s3_bucket.this.id
  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_cloudfront_origin_access_control" "this" {
  name                              = var.bucket_name
  origin_access_control_origin_type = "s3"
  signing_behavior                  = "always"
  signing_protocol                  = "sigv4"
}

data "aws_cloudfront_cache_policy" "optimized" {
  name = "Managed-CachingOptimized"
}

data "aws_cloudfront_cache_policy" "disabled" {
  name = "Managed-CachingDisabled"
}

resource "aws_cloudfront_distribution" "this" {
  enabled             = true
  default_root_object = "index.html"
  aliases             = var.aliases
  price_class         = "PriceClass_200"
  http_version        = "http2and3"

  origin {
    domain_name              = aws_s3_bucket.this.bucket_regional_domain_name
    origin_id                = "s3"
    origin_access_control_id = aws_cloudfront_origin_access_control.this.id
  }

  default_cache_behavior {
    target_origin_id           = "s3"
    viewer_protocol_policy     = "redirect-to-https"
    allowed_methods            = ["GET", "HEAD"]
    cached_methods             = ["GET", "HEAD"]
    cache_policy_id            = data.aws_cloudfront_cache_policy.optimized.id
    response_headers_policy_id = aws_cloudfront_response_headers_policy.this.id
    compress                   = true
  }

  # config.json は環境ごとに差し替えるのでキャッシュしない
  ordered_cache_behavior {
    path_pattern           = "/config.json"
    target_origin_id       = "s3"
    viewer_protocol_policy = "redirect-to-https"
    allowed_methods        = ["GET", "HEAD"]
    cached_methods         = ["GET", "HEAD"]
    cache_policy_id        = data.aws_cloudfront_cache_policy.disabled.id
  }

  # 画面の切り替えは URL を変えないので、存在しないパスを index.html に振り替えない。
  # 振り替えると、置き忘れた config.json や消えた assets/ にも 200 で HTML が返り、原因が見えなくなる

  restrictions {
    geo_restriction {
      restriction_type = "none"
    }
  }

  viewer_certificate {
    acm_certificate_arn      = var.certificate_arn
    ssl_support_method       = "sni-only"
    minimum_protocol_version = "TLSv1.2_2021"
  }
}

locals {
  # 画面が話す相手(API・認証基盤)のオリジン。config.json と同じ値から作るので、ずれない
  connect_origins = distinct(concat(
    [for url in compact([
      var.runtime_config.apiBaseUrl,
      var.runtime_config.oidc.authority,
      var.runtime_config.oidc.endSessionEndpoint,
      var.runtime_config.oidc.revocationEndpoint,
    ]) : regex("^https?://[^/]+", url)],
    var.extra_connect_origins,
  ))
  content_security_policy = join("; ", [
    "default-src 'self'",
    "connect-src 'self' ${join(" ", local.connect_origins)}",
    "img-src 'self' data:",
    "object-src 'none'",
    "base-uri 'self'",
    "form-action 'self'",
    "frame-ancestors 'none'",
  ])
}

# ブラウザに守ってもらう決まり。スクリプトとスタイルは自分のオリジンのファイルだけ
# (ビルド成果物にインラインのスクリプト・スタイルはない)。他のサイトの枠の中には表示させない
resource "aws_cloudfront_response_headers_policy" "this" {
  name = "${var.bucket_name}-security"

  security_headers_config {
    content_security_policy {
      content_security_policy = local.content_security_policy
      override                = true
    }
    strict_transport_security {
      access_control_max_age_sec = 31536000
      include_subdomains         = true
      override                   = true
    }
    content_type_options {
      override = true
    }
    frame_options {
      frame_option = "DENY"
      override     = true
    }
    # ログインから戻った直後の URL には認可コードが載る。他のサイトに Referer で渡さない
    referrer_policy {
      referrer_policy = "same-origin"
      override        = true
    }
  }
}

data "aws_iam_policy_document" "bucket" {
  statement {
    actions   = ["s3:GetObject"]
    resources = ["${aws_s3_bucket.this.arn}/*"]
    principals {
      type        = "Service"
      identifiers = ["cloudfront.amazonaws.com"]
    }
    condition {
      test     = "StringEquals"
      variable = "AWS:SourceArn"
      values   = [aws_cloudfront_distribution.this.arn]
    }
  }
}

resource "aws_s3_bucket_policy" "this" {
  bucket = aws_s3_bucket.this.id
  policy = data.aws_iam_policy_document.bucket.json
}

resource "aws_s3_object" "config" {
  bucket        = aws_s3_bucket.this.id
  key           = "config.json"
  content_type  = "application/json"
  cache_control = "no-store"
  content       = jsonencode(var.runtime_config)
}
