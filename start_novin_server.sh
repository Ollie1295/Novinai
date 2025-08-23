#!/bin/bash
echo "🚀 Starting Novin Heavy AI Server..."
cd /mnt/c/novin
source .venv/bin/activate
echo "📡 Starting FastAPI server on port 8001..."
uvicorn server:app --host 0.0.0.0 --port 8001
