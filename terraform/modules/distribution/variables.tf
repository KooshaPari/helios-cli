variable "environment" {
  description = "The deployment environment"
  type        = string
}

variable "region" {
  description = "The AWS region"
  type        = string
}

variable "bucket_name" {
  description = "The name of the S3 bucket"
  type        = string
}

variable "domain_name" {
  description = "The domain name for CloudFront"
  type        = string
}

variable "price_class" {
  description = "The CloudFront price class"
  type        = string
  default     = "PriceClass_100"
}
