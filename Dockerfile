FROM rust:1.84.0-alpine3.21 AS builder
RUN apk add --no-cache musl-dev openssl-dev libc6-compat
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM alpine:3.21
RUN apk update --no-cache && \
    apk add --no-cache openssl-libs-static && \
    rm -rf /var/cache/apk/*
RUN adduser -D botuser
USER botuser
COPY --from=builder /usr/src/app/target/release/prowlarr-telegram-client /usr/local/bin/
CMD ["/usr/local/bin/prowlarr-telegram-client"]