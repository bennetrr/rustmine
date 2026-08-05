FROM rust:1.97.1-slim-trixie AS builder
WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo install --path rustmine-authentication-service --locked


FROM debian:trixie-slim
RUN useradd appuser
USER appuser
EXPOSE 65432

COPY --from=builder /usr/local/cargo/bin/rustmine-authentication-service /usr/local/bin/rustmine-authentication-service
ENTRYPOINT ["rustmine-authentication-service"]
