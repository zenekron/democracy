locals {
  bot_labels = merge(local.labels, {
    "app.kubernetes.io/name" = "democracy"
    # "app.kubernetes.io/version"   = ""
    "app.kubernetes.io/component" = "discord-bot"
  })
}

# https://registry.terraform.io/providers/hashicorp/kubernetes/latest/docs/resources/secret_v1
resource "kubernetes_secret_v1" "democracy" {
  metadata {
    name      = "democracy"
    namespace = kubernetes_namespace_v1.democracy.metadata[0].name
    labels    = local.bot_labels
  }

  data = {
    DISCORD_TOKEN = var.discord_token
    DATABASE_URL  = local.postgres_url
  }
}

# https://registry.terraform.io/providers/hashicorp/kubernetes/latest/docs/resources/deployment_v1
resource "kubernetes_deployment_v1" "democracy" {
  depends_on = [kubernetes_deployment_v1.postgres]

  metadata {
    name      = "democracy"
    namespace = kubernetes_namespace_v1.democracy.metadata[0].name
    labels    = local.bot_labels
  }

  spec {
    # the bot does not currently support multiple instances running
    replicas = 1

    selector {
      match_labels = local.bot_labels
    }

    template {
      metadata {
        labels = local.bot_labels
      }

      spec {
        container {
          name  = "democracy"
          image = var.democracy_image

          env {
            name  = "RUST_LOG"
            value = "info,democracy=debug"
          }

          env_from {
            prefix = "DEMOCRACY_"

            secret_ref {
              name     = kubernetes_secret_v1.democracy.metadata[0].name
              optional = false
            }
          }
        }

        restart_policy = "Always"
      }
    }
  }
}
