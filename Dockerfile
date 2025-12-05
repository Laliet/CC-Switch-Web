FROM node:20-bookworm AS node-builder
WORKDIR /app
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
RUN corepack enable && pnpm install --frozen-lockfile
COPY tsconfig.json tsconfig.node.json tailwind.config.js vite.config.mts vite.config.web.mts vitest.config.ts ./
COPY src ./src
RUN pnpm build:web

FROM rust:1.83-bookworm AS rust-builder
RUN apt-get update && apt-get install -y --no-install-recommends libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY src-tauri ./src-tauri
COPY --from=node-builder /app/dist-web ./dist-web
WORKDIR /app/src-tauri
RUN cargo build --release --no-default-features --features web-server --example server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*
RUN useradd -m ccswitch
COPY --from=rust-builder /app/src-tauri/target/release/examples/server /usr/local/bin/cc-switch-server
USER ccswitch
WORKDIR /home/ccswitch
ENV HOST=0.0.0.0 PORT=3000
EXPOSE 3000
CMD ["cc-switch-server"]
