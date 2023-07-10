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
