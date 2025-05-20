#!/bin/bash
# Ensure RUST_LOG is set for proper logging
export RUST_LOG=info

# Use port 8000 by default, but allow it to be overridden
# You can run with a different port: PORT=3001 ./run.sh
export PORT=${PORT:-8000}

cd "$(dirname "$0")"
echo "Starting backend server on port $PORT..."
cargo run
