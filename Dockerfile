# Build stage
FROM rust:1-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# Runtime stage
FROM alpine:latest

RUN apk add --no-cache ca-certificates

COPY --from=builder /app/target/release/knaben-api /usr/local/bin/knaben-api

EXPOSE 8080
CMD ["knaben-api"]
