data "docker_registry_image" "brooch" {
    name = "ghcr.io/ryghub/revenants-brooch:latest"
}

resource "docker_image" "brooch" {
    name = data.docker_registry_image.brooch.name

    pull_triggers = [
        data.docker_registry_image.brooch.sha256_digest,
    ]
}

resource "docker_container" "brooch" {
    name = var.docker_container_name
    image = docker_image.brooch.image_id
    restart = "unless-stopped"
}
