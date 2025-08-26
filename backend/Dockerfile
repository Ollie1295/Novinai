# Multi-stage Dockerfile for Insane AI Security Backend
# Optimized for production deployment with security and efficiency

# Stage 1: Base Python image with system dependencies
FROM python:3.11-slim as base

# Set environment variables
ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1 \
    DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && apt-get install -y \
    # Build dependencies
    build-essential \
    curl \
    git \
    # Runtime dependencies  
    libpq-dev \
    libffi-dev \
    libjpeg-dev \
    libpng-dev \
    libwebp-dev \
    # System monitoring
    procps \
    htop \
    # Network utilities
    netcat-openbsd \
    wget \
    # Clean up
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user
RUN groupadd -r insaneai && useradd -r -g insaneai insaneai

# Stage 2: Python dependencies
FROM base as deps

# Set working directory
WORKDIR /app

# Copy requirements
COPY requirements.txt requirements-dev.txt ./

# Install Python dependencies
RUN pip install --no-cache-dir -r requirements.txt \
    && pip install --no-cache-dir -r requirements-dev.txt

# Stage 3: Application code
FROM deps as app

# Copy application code
COPY . .

# Create necessary directories
RUN mkdir -p /app/logs /app/data /app/tmp \
    && chown -R insaneai:insaneai /app

# Switch to non-root user
USER insaneai

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

# Default command (overridden by docker-compose)
CMD ["python", "-m", "uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8000"]

# Stage 4: Development image
FROM app as development

USER root

# Install development tools
RUN pip install --no-cache-dir \
    pytest \
    pytest-asyncio \
    pytest-cov \
    black \
    flake8 \
    mypy \
    pre-commit

USER insaneai

# Development command
CMD ["python", "-m", "uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8000", "--reload"]

# Stage 5: Production image (default)
FROM app as production

# Remove development dependencies
USER root
RUN pip uninstall -y pytest pytest-asyncio pytest-cov black flake8 mypy pre-commit

# Security hardening
RUN chmod -R 750 /app \
    && find /app -name "*.py" -exec chmod 644 {} \; \
    && chmod +x /app/scripts/*.sh || true

USER insaneai

# Production optimizations
ENV PYTHONOPTIMIZE=1

# Multi-process production server
CMD ["python", "-m", "gunicorn", "main:app", "-w", "4", "-k", "uvicorn.workers.UvicornWorker", "--bind", "0.0.0.0:8000"]
