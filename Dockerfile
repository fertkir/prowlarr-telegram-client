FROM rust:1.84.0-alpine3.21 as builder
RUN apk add --no-cache musl-dev openssl-dev libc6-compat openssl-libs-static
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .


FROM alpine:3.21
RUN apk update --no-cache
RUN adduser -D botuser
USER botuser
COPY --from=builder /usr/local/cargo/bin/prowlarr-telegram-client /usr/local/bin/prowlarr-telegram-client
CMD ["prowlarr-telegram-client"]
