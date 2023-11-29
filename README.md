# Telegram bot for [Prowlarr](https://github.com/Prowlarr/Prowlarr)

[![Build Status](https://github.com/fertkir/prowlarr-telegram-client/workflows/CI/badge.svg?branch%3Amain)](https://github.com/fertkir/prowlarr-telegram-client/actions/workflows/ci.yml?query=branch%3Amain)
[![Coverage Status](https://coveralls.io/repos/github/fertkir/prowlarr-telegram-client/badge.svg?branch=main)](https://coveralls.io/github/fertkir/prowlarr-telegram-client?branch=main)

Telegram bot interface for downloading content via Prowlarr.

![Screencast](https://github.com/fertkir/prowlarr-telegram-client/assets/5433737/65898a6a-1316-4be0-a0a4-9239669dd779)

### Configuration

Configuration is done through environment variables.

| Variable              | Description                                                                                                  | Mandatory                            | Default         |
|-----------------------|--------------------------------------------------------------------------------------------------------------|--------------------------------------|-----------------|
| ALLOWED_USERS         | Comma separated list of telegram user ids, who are allowed to use the bot.                                   |                                      | Anyone          |
| COMPLETE_IP           | IP to bind the complete webhook to.                                                                          |                                      | 0.0.0.0         |
| COMPLETE_PORT         | TCP port to listen for download completion requests.                                                         |                                      |                 |
| PROWLARR_API_KEY      | API key to access Prowlarr.                                                                                  | if PROWLARR_API_KEY_FILE isn't set   |                 |
| PROWLARR_API_KEY_FILE | Path to a file with API key to access Prowlarr.                                                              | if PROWLARR_API_KEY isn't set        |                 |
| PROWLARR_BASE_URL     | e.g. http://localhost:9696                                                                                   |                                      |                 |
| REDIS_URL             | Redis URL, to use as a store for link mappings. If not set, a non-persistent in-memory storage will be used. |                                      |                 |
| REDIS_SEQUENCE_START  | First id value to use.                                                                                       |                                      | 1000            |
| REDIS_KEY_EXPIRATION  | When mappings will expire.                                                                                   |                                      | 604800 (1 week) |
| RUST_LOG              | Minimal log level.                                                                                           |                                      | info            |
| TELOXIDE_PROXY        | Proxy to use for connecting to Telegram, e.g. socks5://localhost:9000                                        |                                      |                 |
| TELOXIDE_TOKEN        | Telegram bot token (from [@BotFather](https://t.me/BotFather) bot)                                           | Yes                                  |                 |
| WEBHOOK_IP            | IP to bind the Telegram webhook to.                                                                          |                                      | 0.0.0.0         |
| WEBHOOK_PORT          | Port on which the bot will be listening for requests from Telegram.                                          | For non-polling telegram interaction |                 |
| WEBHOOK_URL           | Example: https://<app-name>.herokuapp.com:443                                                                | For non-polling telegram interaction |                 |

### Usage example

```yaml
# docker-compose.yml

services:
  prowlarr-tg-client:
    image: ghcr.io/fertkir/prowlarr-telegram-client:main
    user: "1000:1000" # TODO replace with your user and group ids
    environment:
      - COMPLETE_PORT=12345
      - PROWLARR_API_KEY=<prowlarr api key> # TODO: replace with your Prowlarr api key
      - PROWLARR_BASE_URL=http://prowlarr:9696
      - RUST_LOG=info
      - TELOXIDE_TOKEN=<telegram token> # TODO: replace with your telegram token
    restart: unless-stopped

  prowlarr:
    image: lscr.io/linuxserver/prowlarr:latest
    environment:
      - PUID=1000  # TODO replace with your user id
      - PGID=1000  # TODO replace with your group id
      - TZ=Etc/UTC
    volumes:
      - prowlarr-config:/config
    ports:
      - "9696:9696"
    restart: unless-stopped

  transmission:
    image: linuxserver/transmission:latest
    restart: unless-stopped
    environment:
      - PUID=1000  # TODO replace with your user id
      - PGID=1000  # TODO replace with your group id
      - TZ=Etc/UTC
      - DOCKER_MODS=ghcr.io/fertkir/prowlarr-tg-client-transmission:main # download-complete callback support
      - PROWLARR_CLIENT_SERVER_URL=http://prowlarr-tg-client:12345       # download-complete callback support
    volumes:
      - transmission-config:/config
      - /home/username/Downloads:/downloads # TODO: replace with your downloads directory
    ports:
      - "9091:9091"
      - "51413:51413"
      - "51413:51413/udp"
volumes:
  prowlarr-config:
  transmission-config:
```
