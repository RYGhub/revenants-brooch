data "docker_registry_image" "brooch" {
    name = "ghcr.io/ryghub/revenants-brooch:latest"
}

resource "docker_image" "brooch" {
    name = data.docker_registry_image.brooch.name

    pull_triggers = [
        data.docker_registry_image.brooch.sha256_digest,
    ]
}

resource "docker_network" "brooch" {
    name = var.docker_network_name
}


resource "docker_container" "brooch" {
    name = var.docker_container_name
    image = docker_image.brooch.image_id
    restart = "unless-stopped"

    env = toset([
        "STRATZ_JWT=${var.stratz_jwt}",
        "DISCORD_WEBHOOK_URL=${var.discord_webhook_url}",
        "FOLLOWED_GUILD_ID=${var.dota_followed_guild_id}",
    ])

    networks_advanced {
        name = docker_network.brooch
    }
}
