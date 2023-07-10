# https://registry.terraform.io/providers/hashicorp/kubernetes/latest/docs/resources/namespace_v1
resource "kubernetes_namespace_v1" "democracy" {
  metadata {
    name   = "democracy"
    labels = local.labels
  }
}
