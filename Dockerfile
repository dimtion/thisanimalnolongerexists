FROM rust:1.70 as builder
WORKDIR /usr/src/wantedspecies
COPY Cargo.toml Cargo.lock .

# Pre-build to cache
RUN cargo install --path . || true

COPY src src 
COPY templates templates
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update & apt-get install -y extra-runtime-dependencies & rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /usr/local/cargo/bin/wantedspecies /usr/local/bin/wantedspecies
COPY static /app/static
COPY database.yml /app/database.yml
CMD ["/usr/local/bin/wantedspecies"]
