terraform {
  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "2.21.1"
    }

    random = {
      source  = "hashicorp/random"
      version = "3.5.1"
    }
  }
}

provider "kubernetes" {}

provider "random" {}

locals {
  # https://kubernetes.io/docs/concepts/overview/working-with-objects/common-labels/
  labels = {
    # "app.kubernetes.io/name"       = ""
    # "app.kubernetes.io/instance"   = ""
    # "app.kubernetes.io/version"    = ""
    # "app.kubernetes.io/component"  = ""
    "app.kubernetes.io/part-of"    = "democracy"
    "app.kubernetes.io/managed-by" = "Terraform"
  }
}

variable "democracy_image" {
  type        = string
  default     = "democracy:latest"
  description = "The container image to use including registry and tag."
}

variable "postgres_image" {
  type        = string
  default     = "postgres:15.3"
  description = "The container image to use for the postgres pods."
}

variable "discord_token" {
  type        = string
  sensitive   = true
  description = "The bot's discord token."
}
