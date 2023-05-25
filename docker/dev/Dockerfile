ARG RUST_BUILDER_IMAGE=clux/muslrust:stable

# Build the torrents.db file
FROM alpine:3 as db_file_builder
RUN apk add sqlite bash
WORKDIR /app
COPY ./data .
WORKDIR /app/scripts
RUN ./import_to_sqlite_fast.sh

FROM $RUST_BUILDER_IMAGE as chef
USER root
RUN cargo install cargo-chef --locked
WORKDIR /app

# Chef plan
FROM chef as planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src src
RUN cargo chef prepare --recipe-path recipe.json

# Chef build
FROM chef as builder
ARG CARGO_BUILD_TARGET=x86_64-unknown-linux-musl
ARG RUSTRELEASEDIR="debug"

COPY --from=planner /app/recipe.json ./recipe.json
RUN cargo chef cook --target ${CARGO_BUILD_TARGET} --recipe-path recipe.json

COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src src
RUN cargo build --target ${CARGO_BUILD_TARGET}

# reduce binary size
RUN strip ./target/$CARGO_BUILD_TARGET/$RUSTRELEASEDIR/torrents-csv-service

RUN cp ./target/$CARGO_BUILD_TARGET/$RUSTRELEASEDIR/torrents-csv-service /app/torrents-csv-service

# The runner
FROM alpine:3
RUN addgroup -S myuser && adduser -S myuser -G myuser
  
# Copy resources
COPY --from=builder /app/torrents-csv-service /app/
COPY --from=db_file_builder /app/torrents.db /app/torrents.db
EXPOSE 8080
USER myuser
CMD ["/app/torrents-csv-service"]
