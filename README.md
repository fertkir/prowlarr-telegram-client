# Telegram bot for [Prowlarr](https://github.com/Prowlarr/Prowlarr)

Telegram bot interface for downloading content via Prowlarr.

![Screencast](https://github.com/fertkir/prowlarr-telegram-client/assets/5433737/65898a6a-1316-4be0-a0a4-9239669dd779)

### Configuration

Configuration is done through environment variables.

| Variable          | Description                                                                                                                          |
|-------------------|--------------------------------------------------------------------------------------------------------------------------------------|
| ALLOWED_USERS     | Optional. Comma separated list of telegram user ids, who are allowed to use the bot. If not set, everyone is allowed to use the bot. |
| COMPLETE_IP       | Optional. IP to bind the complete webhook to. Default: 0.0.0.0                                                                       |
| COMPLETE_PORT     | Optional. TCP port to listen for download completion requests.                                                                       |
| PROWLARR_API_KEY  | API key to access Prowlarr.                                                                                                          |
| PROWLARR_BASE_URL | e.g. http://localhost:9696                                                                                                           |
| RUST_LOG          | Minimal log level, e.g. info                                                                                                         |
| TELOXIDE_PROXY    | Optional. Proxy to use for connecting to Telegram, e.g. socks5://localhost:9000                                                      |
| TELOXIDE_TOKEN    | Telegram bot token (from [@BotFather](https://t.me/BotFather) bot)                                                                   |
| WEBHOOK_IP        | Optional. IP to bind the Telegram webhook to. Default: 0.0.0.0                                                                       |
| WEBHOOK_PORT      | Optional if polling interaction is okay for you. Port on which the bot will be listening for requests from Telegram.                 |
| WEBHOOK_URL       | Optional if polling interaction is okay for you. Example: https://<app-name>.herokuapp.com:443                                       |

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
