provider "aws" {
  region = "ap-northeast-1"
  default_tags {
    tags = {
      Project     = "platform"
      Environment = "dev"
      ManagedBy   = "terraform"
    }
  }
}
