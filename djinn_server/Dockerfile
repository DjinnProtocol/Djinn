# Needs to run from the root (parent) of the project
FROM rust:1.67.0 AS builder
COPY . .
WORKDIR /djinn_server
RUN cargo build --release
RUN echo "serving_directory: /files" > config.yaml

FROM alpine:latest

# Set time zone to CEST
RUN apk add --no-cache tzdata
ENV TZ=Europe/Amsterdam

COPY --from=builder /djinn_server/target/release/djinn_server .
COPY --from=builder /djinn_server/config.yaml .
RUN mkdir files

CMD ["./djinn_server"]
