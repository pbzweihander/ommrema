# syntax = docker/dockerfile:1
FROM rust:1.82-slim AS rustbase
WORKDIR /app


FROM node:20-bookworm-slim AS nodebase
WORKDIR /app


FROM nodebase AS frontend-builder
WORKDIR /app
COPY package.json ./package.json
COPY frontend/package.json ./frontend/package.json
COPY yarn.lock ./yarn.lock
RUN yarn install --frozen-lockfile
COPY frontend ./
RUN yarn build


FROM rustbase AS backend-builder
COPY Cargo.lock .
COPY Cargo.toml .
COPY backend backend
COPY --from=frontend-builder /app/dist /app/frontend/dist
RUN cargo build --release


FROM debian:bookworm-slim AS runtime
RUN apt update \
    && apt install -y \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*
COPY --from=backend-builder /app/target/release/ommrema /usr/local/bin

CMD ["ommrema"]
