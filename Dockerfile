# ── Stage 1: Build Rust backend ───────────────────────────────────────────────
FROM rust:1-bookworm AS backend-builder
WORKDIR /app/backend

# Copy manifests first so cargo can cache the dependency-compilation layer.
# The dummy main.rs keeps the crate valid without the real source.
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release || true

# Now copy real source and force a rebuild of the final binary.
COPY backend/src ./src
COPY backend/migrations ./migrations
# Touch main.rs so cargo knows the crate root has changed, otherwise the
# cached artifact from the dummy build above may be reused.
RUN touch src/main.rs && cargo build --release


# ── Stage 2: Build SvelteKit frontend ─────────────────────────────────────────
FROM node:22-bookworm AS frontend-builder
WORKDIR /app/frontend

COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build


# ── Stage 3: Runtime ──────────────────────────────────────────────────────────
# Use the Node image so we can run the SvelteKit Node server alongside Axum.
FROM node:22-bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libsqlite3-0 curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled Rust binary.
COPY --from=backend-builder /app/backend/target/release/pawtal ./pawtal

# Copy the SvelteKit build output and its package.json (needed by adapter-node
# to resolve the server entry point and its own dependencies).
COPY --from=frontend-builder /app/frontend/build ./frontend/build
COPY --from=frontend-builder /app/frontend/node_modules ./frontend/node_modules
COPY --from=frontend-builder /app/frontend/package.json ./frontend/package.json

# Copy the entrypoint script.
COPY docker/entrypoint.sh ./entrypoint.sh
RUN chmod +x ./entrypoint.sh

# Create a non-root user and set up writable directories.
RUN groupadd -r pawtal \
    && useradd -r -g pawtal -d /app -s /sbin/nologin pawtal \
    && mkdir -p data uploads \
    && chown -R pawtal:pawtal /app/data /app/uploads ./pawtal ./entrypoint.sh

USER pawtal

# ── Environment defaults ───────────────────────────────────────────────────────
# These can all be overridden at runtime via docker-compose environment or
# `docker run -e`. Values here are sensible container-local defaults.
ENV DATABASE_URL=sqlite:/app/data/pawtal.db?mode=rwc
ENV UPLOADS_DIR=/app/uploads
ENV PORT=8080
# ORIGIN is required by SvelteKit's CSRF protection. Set it to the public URL
# of the application; overridden in docker-compose via BASE_URL.
ENV ORIGIN=http://localhost:8080
ENV FRONTEND_ORIGIN=http://127.0.0.1:3000

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD curl -sf http://localhost:8080/api/health || exit 1

ENTRYPOINT ["./entrypoint.sh"]
