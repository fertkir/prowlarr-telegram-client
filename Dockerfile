FROM rust:1.73 as builder
WORKDIR /usr/src/prowlarr-telegram-client
COPY . .
RUN cargo install --path .

FROM debian:stable-slim

ARG REPO
ARG REVISION
LABEL org.opencontainers.image.source="${REPO}" \
    org.opencontainers.image.revision="${REVISION}"

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/prowlarr-telegram-client /usr/local/bin/prowlarr-telegram-client
CMD ["prowlarr-telegram-client"]
