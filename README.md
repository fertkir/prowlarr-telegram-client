# Telegram bot for Prowlarr

### Configuration

Configuration is done through environment variables.

| Variable           | Description                                                                                                                          |
|--------------------|--------------------------------------------------------------------------------------------------------------------------------------|
| ALLOWED_USERS      | Optional. Comma separated list of telegram user ids, who are allowed to use the bot. If not set, everyone is allowed to use the bot. |
| COMPLETE_PORT      | Optional. TCP port to listen for download completion requests.                                                                       |
| PROWLARR_API_KEY   | API key to access Prowlarr.                                                                                                          |
| PROWLARR_BASE_URL  | e.g. http://localhost:9696                                                                                                           |
| RUST_LOG           | Minimal log level, e.g. info                                                                                                         |
| TELOXIDE_TOKEN     | Telegram bot token (from [@BotFather](https://t.me/BotFather) bot)                                                                   |
| WEBHOOK_PORT       | Optional if polling interaction is okay for you. Port on which the bot will be listening for requests from Telegram.                 |
| WEBHOOK_URL        | Optional if polling interaction is okay for you. Example: https://<app-name>.herokuapp.com:443                                       |
