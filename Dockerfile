FROM rust:1-buster AS build

WORKDIR /work
COPY . .
RUN cargo build --release

FROM debian:buster

COPY --from=build /workdir/target/release/t /usr/bin/t
