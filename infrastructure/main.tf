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

# https://registry.terraform.io/providers/hashicorp/kubernetes/latest/docs/resources/namespace_v1
resource "kubernetes_namespace_v1" "democracy" {
  metadata {
    name   = "democracy"
    labels = local.labels
  }
}
