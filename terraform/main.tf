terraform {
  required_version = ">= 1.0.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
  backend "s3" {
    bucket         = "my-terraform-state-bucket"
    key            = "helios-cli/terraform.tfstate"
    region         = "us-east-1"
    dynamodb_table = "my-terraform-lock-table"
    encrypt        = true
  }
}

provider "aws" {
  alias  = "us_east_1"
  region = "us-east-1"
}

provider "aws" {
  alias  = "eu_west_1"
  region = "eu-west-1"
}

provider "aws" {
  alias  = "ap_southeast_1"
  region = "ap-southeast-1"
}

module "us_east_1_distribution" {
  source = "./modules/distribution"
  providers = {
    aws = aws.us_east_1
  }
  environment       = var.environment
  region            = "us-east-1"
  bucket_name       = "helios-cli-dist-us-east-1"
  domain_name       = var.domain_name
}

module "eu_west_1_distribution" {
  source = "./modules/distribution"
  providers = {
    aws = aws.eu_west_1
  }
  environment       = var.environment
  region            = "eu-west-1"
  bucket_name       = "helios-cli-dist-eu-west-1"
  domain_name       = var.domain_name
}

module "ap_southeast_1_distribution" {
  source = "./modules/distribution"
  providers = {
    aws = aws.ap_southeast_1
  }
  environment       = var.environment
  region            = "ap-southeast-1"
  bucket_name       = "helios-cli-dist-ap-southeast-1"
  domain_name       = var.domain_name
}

# Lambda@Edge for Geo-Routing
resource "aws_lambda_function" "geo_router" {
  function_name = "helios-cli-geo-router"
  runtime       = "nodejs14.x"
  handler       = "index.handler"
  role          = aws_iam_role.lambda_edge.arn

  filename      = "lambda/geo_router.zip" # Placeholder
  source_code_hash = filebase64sha256("lambda/geo_router.zip")
}

resource "aws_iam_role" "lambda_edge" {
  name = "helios-cli-lambda-edge-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action = "sts:AssumeRole"
      Effect = "Allow"
      Principal = {
        Service = [
          "lambda.amazonaws.com",
          "edgelambda.amazonaws.com"
        ]
      }
    }]
  })
}
