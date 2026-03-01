FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# Avoid prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies + Node.js 20 via NodeSource
RUN apt-get update && apt-get install -y --no-install-recommends \
    binaryen \
    wabt \
    pkg-config \
    libssl-dev \
    curl \
    && curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js global tools
RUN npm install -g wrangler esbuild

# Switch to vscode user for cargo installs
USER vscode

# Add wasm target
RUN rustup target add wasm32-unknown-unknown

# Install Rust CLI tools
RUN cargo install wasm-pack cargo-watch && \
    cargo install --locked cargo-nextest

WORKDIR /workspaces/transfigure
