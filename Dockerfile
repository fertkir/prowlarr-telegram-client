FROM rust:1.74-alpine3.18 as builder
RUN apk add --no-cache musl-dev openssl-dev libc6-compat openssl-libs-static
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .

FROM alpine:3.18
RUN apk update
COPY --from=builder /usr/local/cargo/bin/prowlarr-telegram-client /usr/local/bin/prowlarr-telegram-client
CMD ["prowlarr-telegram-client"]
