variable "environment" {
  description = "The deployment environment"
  type        = string
}

variable "region" {
  description = "The AWS region"
  type        = string
}

variable "vpc_cidr" {
  description = "The CIDR block for the VPC"
  type        = string
}

variable "service_name" {
  description = "The name of the service"
  type        = string
  default     = "helios-cli"
}

variable "container_port" {
  description = "The port the container listens on"
  type        = number
  default     = 8080
}
