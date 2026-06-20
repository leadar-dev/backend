FROM rust:1.88-alpine AS builder
RUN apk add --no-cache musl-dev openssl-dev pkgconfig
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
# cache deps layer
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx
ENV SQLX_OFFLINE=true
RUN touch src/main.rs && cargo build --release

FROM alpine:3.21
RUN apk add --no-cache ca-certificates libgcc
WORKDIR /app
COPY --from=builder /app/target/release/backend ./backend
COPY migrations ./migrations
ENV RUST_LOG=info
CMD ["./backend"]
