# Multi-stage build for Rust binaries
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY mcp-common ./mcp-common
COPY mcp-proxy ./mcp-proxy  
COPY mcp-monitor ./mcp-monitor

# Build release binaries
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -u 1000 app

# Create directories
RUN mkdir -p /app/logs /tmp/mcp-sockets && \
    chown -R app:app /app /tmp/mcp-sockets

# Copy binaries from builder
COPY --from=builder /app/target/release/mcp-proxy /usr/local/bin/
COPY --from=builder /app/target/release/mcp-monitor /usr/local/bin/

# Switch to app user
USER app

# Default working directory
WORKDIR /app

# Environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Default to running the monitor
CMD ["mcp-monitor"]