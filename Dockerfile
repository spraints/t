FROM rust:1-buster AS build

WORKDIR /work
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY src        src
RUN cargo build --release

FROM debian:buster-slim

COPY public /usr/share/t/wwwroot
COPY --from=build /work/target/release/t /usr/bin/t
