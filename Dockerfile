FROM rust:1.80.1-slim as builder

ENV APP_DIR=/usr/src/app

WORKDIR ${APP_DIR}
COPY src/ ./src
COPY Cargo.toml Cargo.lock ./

RUN cargo build --release

FROM gcr.io/distroless/cc-debian12 as release
WORKDIR /app
COPY --from=builder /usr/src/app/target/release/axum-demo ./axum_demo

EXPOSE 8080

CMD ["./axum_demo"]
