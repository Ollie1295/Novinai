#!/bin/bash

echo "🚀 Starting GPU Worker..."

# Activate virtual environment
source gpu_worker_env/bin/activate

echo "✅ Virtual environment activated"

# Check if CUDA is available
python3 -c "import torch; print(f'CUDA Available: {torch.cuda.is_available()}')"

# Run the GPU worker
echo "🔥 Starting GPU Worker Script..."
python3 gpu_worker_simple.py

echo "✅ GPU Worker stopped"
