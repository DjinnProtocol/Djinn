# Needs to run from the root (parent) of the project
FROM rust:1.67.0 AS builder
COPY . .
WORKDIR /djinn_server
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN echo "serving_directory: /files" > config.yaml
RUN pwd
RUN ls -lr target/release

FROM alpine:latest

# Set time zone to CEST
RUN apk add --no-cache tzdata
ENV TZ=Europe/Amsterdam

COPY --from=builder /djinn_server/target/x86_64-unknown-linux-musl/release/djinn_server .
COPY --from=builder /djinn_server/config.yaml .
RUN mkdir files

CMD /djinn_server
