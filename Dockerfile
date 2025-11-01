FROM rust:1.90.0-alpine3.22 AS builder
RUN apk add --no-cache build-base libstdc++-dev openssl-dev
WORKDIR /usr/src/app
COPY . .
ENV OPENSSL_STATIC=true
ENV RUSTFLAGS='-C target-feature=-crt-static'
RUN cargo build --release

FROM alpine:3.22
RUN apk add --no-cache libgcc libc6-compat
RUN addgroup -g 1000 appgroup \
    && adduser -u 1000 -G appgroup -s /bin/sh -D appuser
COPY --from=builder /usr/src/app/target/release/prowlarr-telegram-client /usr/local/bin/
RUN chown appuser:appgroup /usr/local/bin/prowlarr-telegram-client
USER appuser
ENTRYPOINT ["/usr/local/bin/prowlarr-telegram-client"]