# Multi-stage Dockerfile for Context Engine MCP Server (Rust)
#
# Build: docker build -t context-engine .
# Run:   docker run -v /path/to/project:/workspace context-engine --workspace /workspace
#
# Image size: ~15 MB (Alpine-based minimal image)

# Stage 1: Build the Rust binary with musl for static linking
FROM rust:1.83-alpine AS builder

# Install build dependencies for static linking
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

WORKDIR /app

# Copy everything needed for build
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build statically linked release binary
ENV OPENSSL_STATIC=1
RUN cargo build --release

# Stage 2: Minimal Alpine runtime (~5MB base)
FROM alpine:3.19

# Install only CA certificates for HTTPS
RUN apk add --no-cache ca-certificates

# Create non-root user
RUN adduser -D -u 1000 context-engine

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/context-engine /app/context-engine

# Set ownership
RUN chown -R context-engine:context-engine /app

# Switch to non-root user
USER context-engine

# Default environment variables
ENV CONTEXT_ENGINE_TRANSPORT=stdio
ENV CONTEXT_ENGINE_PORT=3000

# Expose HTTP port (only used with --transport http)
EXPOSE 3000
EXPOSE 9090

# Default command (can be overridden)
ENTRYPOINT ["/app/context-engine"]
CMD ["--help"]

