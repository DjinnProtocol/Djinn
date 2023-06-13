# Needs to run from the root (parent) of the project
FROM rust:1.67.0 AS builder
COPY . .
WORKDIR /djinn_server
RUN cargo build --release
RUN echo "serving_directory: /files" > config.yaml
RUN rm ./files/.gitignore

FROM gcr.io/distroless/cc
COPY --from=builder /djinn_server/target/release/djinn_server /
COPY --from=builder /djinn_server/config.yaml /
COPY --from=builder /djinn_server/files /files

CMD ["./djinn_server"]
