# Needs to run from the root (parent) of the project
FROM rust:1.67.0 AS builder
COPY . .
WORKDIR /djinn_client_cli
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=builder /djinn_client_cli/target/release/djinn_client_cli /
CMD ["./djinn_client_cli"]
