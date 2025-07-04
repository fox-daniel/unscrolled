# Unscrolled Backend

A simple backend server for the Unscrolled chat application.

## Features

- Health check endpoint at `/health`
- Built with Axum web framework
- CORS enabled for frontend integration

## Getting Started

### Prerequisites

- Rust and Cargo installed

### Running the Server

You can run the server using the provided shell script:

```bash
# Make the script executable first
chmod +x run.sh
./run.sh
```

By default, the server starts on port 8000. To use a different port:

```bash
# Specify a different port
PORT=8001 ./run.sh
```

Or manually with cargo:

```bash
cd backend
PORT=8001 cargo run
```

The server will start on http://127.0.0.1:8000 (or your specified port)


### Testing the Health Check

Once the server is running, you can test the health check endpoint:

```bash
curl http://127.0.0.1:8000/health
```

Expected response:
```json
{
  "status": "ok",
  "message": "Unscrolled API is running",
  "version": "0.1.0"
}
```

## Next Steps

- Add API endpoints for chat messages
- Implement WebSocket support for real-time chat
- Integrate with the frontend