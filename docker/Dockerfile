ARG RUST_VERSION=1.72
ARG RUST_RELEASE_MODE=debug
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
ARG RUST_RELEASE_MODE

COPY --from=planner /app/recipe.json ./recipe.json

# Cargo chef cook
RUN set -ex; \
  if [ "${RUST_RELEASE_MODE}" = "debug" ]; then \
    cargo chef cook --recipe-path recipe.json; \
  else \
    cargo chef cook --release --recipe-path recipe.json; \
  fi

COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src src

# Cargo build
RUN set -ex; \
  if [ "${RUST_RELEASE_MODE}" = "debug" ]; then \
    cargo build; \
  else \
    cargo build --release; \
  fi

# reduce binary size
RUN strip ./target/$RUST_RELEASE_MODE/torrents-csv-service

RUN cp ./target/$RUST_RELEASE_MODE/torrents-csv-service /app/torrents-csv-service

# The runner
FROM $DISTROLESS_IMAGE
  
# Copy resources
COPY --from=builder /app/torrents-csv-service /app/
COPY --from=db_file_builder /app/torrents.db /app/torrents.db
EXPOSE 8080
CMD ["/app/torrents-csv-service"]
