#!/bin/sh
set -e

# ── Entrypoint for the Pawtal single-container deployment ─────────────────────
#
# Architecture inside the container:
#   • SvelteKit (adapter-node) handles SSR on port 3000 — required for
#     server-side auth checks in the admin layout.
#   • Axum handles all /api/* and /uploads/* routes on port 8080 and reverse-
#     proxies every other request to the SvelteKit server on port 3000.
#   • Only port 8080 is published to the host.

# Pass the public base URL through to SvelteKit as ORIGIN so its built-in CSRF
# protection accepts requests. Falls back to BASE_URL if ORIGIN is unset.
export ORIGIN="${ORIGIN:-${BASE_URL:-http://localhost:8080}}"

echo "Starting SvelteKit Node server on port 3000..."
cd /app/frontend && PORT=3000 node build/index.js &
SVELTEKIT_PID=$!

echo "Starting Axum API server on port 8080..."
cd /app && exec ./pawtal &
AXUM_PID=$!

# Forward SIGTERM/SIGINT to both children so Docker stop is clean.
trap 'kill $SVELTEKIT_PID $AXUM_PID 2>/dev/null; wait' TERM INT

# Wait for either child to exit. If one dies, bring down the other so Docker
# can detect the failure and restart the container.
wait -n 2>/dev/null || wait
echo "A child process exited — shutting down container."
kill $SVELTEKIT_PID $AXUM_PID 2>/dev/null
wait
