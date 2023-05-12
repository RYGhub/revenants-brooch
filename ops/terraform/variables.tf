variable "docker_container_name" {
    type        = string
    description = "Name of the container to create"
    nullable    = false
}

variable "docker_network_name" {
    type        = string
    description = "Name of the network to create"
    nullable    = false
}

variable "stratz_jwt" {
    type        = string
    description = "Stratz JSON Web Token <https://stratz.com/api>"
    nullable    = false
    sensitive   = true
}

variable "discord_webhook_url" {
    type        = string
    description = "Discord webhook URL where messages should be sent in"
    nullable    = false
    sensitive   = true
}

variable "dota_followed_guild_id" {
    type        = string
    description = "ID of the guild to post updates of"
    nullable    = false
}
