# CIM Infrastructure Docker Image
# Multi-stage build for optimized container size

# Build stage
FROM rust:1.75-slim as builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY cim-domain-infrastructure ./cim-domain-infrastructure/
COPY cim-infrastructure-nats ./cim-infrastructure-nats/

# Build release binaries
RUN cargo build --release --workspace

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 cim && \
    mkdir -p /data && \
    chown -R cim:cim /data

WORKDIR /app

# Copy binaries from builder (if any exist)
COPY --from=builder /build/target/release/cim-* /app/ || true

# Copy documentation and examples
COPY --from=builder /build/cim-domain-infrastructure/examples /app/examples/
COPY README.md STATUS.md /app/

# Set ownership
RUN chown -R cim:cim /app

# Switch to non-root user
USER cim

# Environment variables
ENV RUST_LOG=info
ENV NATS_URL=nats://localhost:4222

# Metadata
LABEL org.opencontainers.image.title="CIM Infrastructure"
LABEL org.opencontainers.image.description="Event-sourced infrastructure management for CIM"
LABEL org.opencontainers.image.vendor="Cowboy AI, LLC"
LABEL org.opencontainers.image.licenses="MIT OR Apache-2.0"

# Default command
CMD ["sh", "-c", "echo 'CIM Infrastructure container ready. No default binary - this is a library workspace.'"]
