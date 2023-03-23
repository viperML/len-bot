job "len-bot" {
  datacenters = ["dc1"]

  type = "service"

  group "main-group" {
    count = 1

    network {
      mode = "bridge"
      dns {
        servers = [
          "8.8.8.8",
          "8.8.4.4"
        ]
        options = [
          "edns0",
          "trust-ad"
        ]
      }
    }

    task "run" {
      driver = "docker"
      restart {
        attempts = 0
      }

      vault {
        policies = ["len-bot"]
      }

      template {
        data        = <<EOF
          DISCORD_TOKEN="{{with secret "kv/data/len-bot"}}{{.Data.data.DISCORD_TOKEN}}{{end}}"
          OPENAI_API_KEY="{{with secret "kv/data/len-bot"}}{{.Data.data.OPENAI_API_KEY}}{{end}}"
        EOF
        env         = true
        destination = "secrets/login.env"
      }

      config {
        nix_flake_ref = "github:viperML/len-bot/${var.rev}#default"
        nix_flake_sha = var.narHash
        entrypoint = [
          "bin/len-bot",
        ]
      }

      resources {
        cpu    = 500
        memory = 256
      }
    }
  }
}

variable "rev" {
  type = string
  validation {
    condition     = var.rev != "null"
    error_message = "Git tree is dirty."
  }
}

variable "narHash" {
  type = string
}
