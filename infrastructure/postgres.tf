locals {
  postgres_labels = merge(local.labels, {
    "app.kubernetes.io/name" = "postgres"
    # "app.kubernetes.io/version"   = "" # TODO: here
    "app.kubernetes.io/component" = "database"
  })

  postgres_user     = "democracy"
  postgres_password = random_password.postgres_password.result
  postgres_host     = "${kubernetes_service_v1.postgres.metadata[0].name}.${kubernetes_service_v1.postgres.metadata[0].namespace}.svc.cluster.local"
  postgres_port     = 5432
  postgres_database = "democracy"
  postgres_url      = "postgresql://${local.postgres_user}:${local.postgres_password}@${local.postgres_host}:${local.postgres_port}/${local.postgres_database}"
}

# https://registry.terraform.io/providers/hashicorp/random/latest/docs/resources/password
resource "random_password" "postgres_password" {
  length  = 16
  special = false
}

# https://registry.terraform.io/providers/hashicorp/kubernetes/latest/docs/resources/secret_v1
resource "kubernetes_secret_v1" "postgres_credentials" {
  metadata {
    name      = "postgres-credentials"
    namespace = kubernetes_namespace_v1.democracy.metadata[0].name
    labels    = local.postgres_labels
  }

  data = {
    USER     = local.postgres_user
    PASSWORD = local.postgres_password
  }
}

# https://registry.terraform.io/providers/hashicorp/kubernetes/latest/docs/resources/persistent_volume_claim_v1
resource "kubernetes_persistent_volume_claim_v1" "postgres_data" {
  metadata {
    name      = "postgres-data"
    namespace = kubernetes_namespace_v1.democracy.metadata[0].name
    labels    = local.postgres_labels
  }

  spec {
    access_modes = ["ReadWriteOnce"]

    resources {
      requests = {
        storage = "4Gi"
      }
    }
  }

  wait_until_bound = false
}

# https://registry.terraform.io/providers/hashicorp/kubernetes/latest/docs/resources/deployment_v1
resource "kubernetes_deployment_v1" "postgres" {
  metadata {
    name      = "postgres"
    namespace = kubernetes_namespace_v1.democracy.metadata[0].name
    labels    = local.postgres_labels
  }

  spec {
    # the bot does not currently support multiple instances running
    replicas = 1

    selector {
      match_labels = local.postgres_labels
    }

    template {
      metadata {
        labels = local.postgres_labels
      }

      spec {
        container {
          name  = "postgres"
          image = "postgres:15.3"

          env {
            name  = "POSTGRES_DB"
            value = local.postgres_database
          }

          env_from {
            prefix = "POSTGRES_"

            secret_ref {
              name     = kubernetes_secret_v1.postgres_credentials.metadata[0].name
              optional = false
            }
          }

          port {
            name           = "postgres"
            protocol       = "TCP"
            container_port = local.postgres_port
          }

          volume_mount {
            name       = "postgres-data"
            mount_path = "/var/lib/postgresql/data"
          }
        }

        restart_policy = "Always"

        volume {
          name = "postgres-data"
          persistent_volume_claim {
            claim_name = kubernetes_persistent_volume_claim_v1.postgres_data.metadata[0].name
          }
        }
      }
    }
  }
}

# https://registry.terraform.io/providers/hashicorp/kubernetes/latest/docs/resources/service_v1
resource "kubernetes_service_v1" "postgres" {
  metadata {
    name      = "postgres"
    namespace = kubernetes_namespace_v1.democracy.metadata[0].name
    labels    = local.postgres_labels
  }

  spec {
    type     = "ClusterIP"
    selector = local.postgres_labels

    port {
      name        = "postgres"
      protocol    = "TCP"
      port        = local.postgres_port
      target_port = local.postgres_port
    }
  }
}
