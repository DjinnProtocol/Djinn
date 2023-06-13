# Needs to run from the root (parent) of the project
FROM rust:1.67.0 AS builder
COPY . .
WORKDIR /djinn_server
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=builder /djinn_server/target/release/djinn_server /
RUN echo "serving_directory: /files" > config.yaml
RUN mkdir /files
CMD ["./djinn_server"]
