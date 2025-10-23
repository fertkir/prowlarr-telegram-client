FROM rust:1.90.0-alpine3.22 AS builder
RUN apk add --no-cache build-base libstdc++-dev openssl-dev
WORKDIR /usr/src/app
COPY . .
ENV OPENSSL_STATIC=true
ENV RUSTFLAGS='-C target-feature=-crt-static'
RUN cargo build --release

FROM alpine:3.22
RUN apk add --no-cache libc6-compat
COPY --from=builder /usr/src/app/target/release/prowlarr-telegram-client /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/prowlarr-telegram-client"]