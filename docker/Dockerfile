ARG RUST_VERSION=1.73
ARG DISTROLESS_IMAGE=gcr.io/distroless/cc-debian12

# Build the torrents.db file
FROM alpine as db_file_builder
RUN apk add sqlite bash
WORKDIR /app
COPY ./data .
WORKDIR /app/scripts
RUN ./import_to_sqlite_fast.sh

FROM lukemathwalker/cargo-chef:latest-rust-${RUST_VERSION} as chef
WORKDIR /app

FROM chef as planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src src
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json ./recipe.json

# Cargo chef cook
RUN cargo chef cook --release --recipe-path recipe.json

COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src src

# Cargo build
RUN cargo build --release

# reduce binary size
RUN strip ./target/release/torrents-csv-service

RUN cp ./target/release/torrents-csv-service /app/torrents-csv-service

# The runner
FROM $DISTROLESS_IMAGE

# Copy resources
COPY --from=builder /app/torrents-csv-service /app/
COPY --from=db_file_builder /app/torrents.db /app/torrents.db
EXPOSE 8080
CMD ["/app/torrents-csv-service"]
