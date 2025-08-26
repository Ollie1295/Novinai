# Insane AI Security - Deployment Guide

## Overview

This document provides comprehensive instructions for deploying the Insane AI Security system using Docker and Docker Compose. The system is designed for both development and production environments with full monitoring, security, and scalability features.

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│      Nginx      │    │   Load Balancer │    │      CDN        │
│  (Reverse Proxy)│◄───┤   (Optional)    │◄───┤   (Optional)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
          │
          ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   API Gateway   │    │   API Gateway   │    │   API Gateway   │
│  (FastAPI x4)   │    │  (FastAPI x4)   │    │  (FastAPI x4)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
          │                      │                      │
          └──────────────────────┼──────────────────────┘
                                 ▼
                    ┌─────────────────┐
                    │   Redis Cluster │
                    │ (Message Broker)│
                    └─────────────────┘
                                 │
          ┌──────────────────────┼──────────────────────┐
          ▼                      ▼                      ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Scheduler     │    │  Deep Worker    │    │  Deep Worker    │
│   (x1)          │    │    (x6)         │    │    (x6)         │
└─────────────────┘    └─────────────────┘    └─────────────────┘

Monitoring Stack:
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Prometheus    │◄───┤     Grafana     │◄───┤  Alertmanager   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
          ▲
          │
┌─────────────────┐    ┌─────────────────┐
│ Redis Exporter  │    │    Promtail     │
└─────────────────┘    └─────────────────┘
```

## Quick Start

### Prerequisites

- Docker 24.0+ and Docker Compose 2.0+
- At least 8GB RAM (16GB recommended for production)
- 50GB free disk space
- Git

### Development Environment

```bash
# Clone repository
git clone https://github.com/insane-ai/security.git
cd security

# Initialize project
make init

# Start development environment
make dev

# View logs
make dev-logs

# Check status
make status
```

**Development URLs:**
- API: http://localhost:8000 
- API Docs: http://localhost:8000/docs
- Grafana: http://localhost:3000 (admin/admin)
- Prometheus: http://localhost:9090
- Redis Commander: http://localhost:8081

### Production Deployment

```bash
# Setup production secrets
make setup-secrets

# Deploy to production
make prod-deploy

# Check status
make status
```

## Detailed Setup

### 1. Environment Configuration

Create `.env` file:

```env
# Production Settings
JWT_SECRET=your-super-secret-jwt-key-here
API_KEY=your-api-key-here
GRAFANA_PASSWORD=your-secure-grafana-password

# Infrastructure
NUM_GPUS=2
SLACK_WEBHOOK_URL=https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK

# Domains (Production)
API_DOMAIN=api.insane.ai
GRAFANA_DOMAIN=grafana.insane.ai
PROMETHEUS_DOMAIN=prometheus.insane.ai
```

### 2. SSL Certificate Setup

For production, place SSL certificates in:
```
ssl/
├── certs/
│   ├── api.insane.ai.crt
│   ├── grafana.insane.ai.crt
│   └── prometheus.insane.ai.crt
└── private/
    ├── api.insane.ai.key
    ├── grafana.insane.ai.key
    └── prometheus.insane.ai.key
```

### 3. Docker Swarm (Production)

Initialize Docker Swarm for production scaling:

```bash
# Initialize swarm
docker swarm init

# Label nodes
docker node update --label-add gpu=true NODE_ID
docker node update --label-add storage=ssd NODE_ID
docker node update --label-add monitoring=true NODE_ID
docker node update --label-add database=true NODE_ID

# Deploy stack
docker stack deploy -c docker-compose.yml -c docker-compose.prod.yml insane-ai
```

## Service Details

### Core Services

#### API Gateway
- **Purpose**: REST API for mobile clients
- **Technology**: FastAPI with Uvicorn/Gunicorn
- **Scaling**: 4 replicas (adjust based on load)
- **Health Check**: `/health`
- **Metrics**: Prometheus metrics on `/metrics`

#### Scheduler
- **Purpose**: Event processing orchestration
- **Technology**: Custom Python service
- **Scaling**: Single instance (stateful)
- **Features**: Token bucket rate limiting, autothrottle

#### Deep Worker
- **Purpose**: ML-based event processing
- **Technology**: PyTorch/TensorFlow workers
- **Scaling**: 2-6 replicas (GPU dependent)
- **Features**: Batch processing, session-based jobs

#### Redis
- **Purpose**: Message broker and cache
- **Configuration**: RDB + AOF persistence
- **Scaling**: Single instance with clustering option
- **Monitoring**: Redis Exporter

### Monitoring Services

#### Prometheus
- **Purpose**: Metrics collection and alerting
- **Retention**: 90 days (production), 7 days (dev)
- **Storage**: 50GB limit
- **Scrape Interval**: 15 seconds

#### Grafana
- **Purpose**: Visualization and dashboards
- **Authentication**: Admin user + LDAP (optional)
- **Plugins**: Redis datasource
- **Session Storage**: Redis

#### Alertmanager
- **Purpose**: Alert routing and notifications
- **Integrations**: Slack, PagerDuty, email
- **Grouping**: By service and severity

## Configuration Files

### Makefile Commands

| Command | Description |
|---------|-------------|
| `make dev` | Start development environment |
| `make prod` | Start production environment |
| `make build` | Build all services |
| `make test` | Run test suite |
| `make backup` | Backup production data |
| `make scale-api REPLICAS=6` | Scale API service |
| `make clean` | Clean up containers |

### Docker Compose Files

- `docker-compose.yml` - Base configuration
- `docker-compose.dev.yml` - Development overrides  
- `docker-compose.prod.yml` - Production overrides

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ENVIRONMENT` | development | Runtime environment |
| `REDIS_URL` | redis://redis:6379 | Redis connection string |
| `JWT_SECRET` | - | JWT signing secret |
| `NUM_GPUS` | 1 | Number of GPUs available |
| `LOG_LEVEL` | INFO | Logging level |

## Security Considerations

### Network Security
- All services run in isolated Docker network
- Only necessary ports exposed
- Nginx reverse proxy with rate limiting

### Application Security
- JWT token authentication
- Signed URLs for image access
- Rate limiting per user/IP
- CORS protection
- Security headers (HSTS, CSP, etc.)

### Infrastructure Security
- Non-root containers
- Secret management with Docker secrets
- File permission restrictions
- Regular security updates via Watchtower

### Monitoring Security
- Prometheus authentication
- Grafana HTTPS with secure cookies
- Alert notifications encrypted

## Scaling Guidelines

### Horizontal Scaling

**API Service:**
```bash
# Scale based on CPU/memory usage
make scale-api REPLICAS=6
```

**Deep Workers:**
```bash
# Scale based on queue backlog
make scale-worker REPLICAS=10
```

### Vertical Scaling

Update resource limits in compose files:
```yaml
deploy:
  resources:
    limits:
      memory: 4G
      cpus: '2.0'
```

### Auto-scaling

For Kubernetes deployment, use HPA:
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: api-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
```

## Backup and Recovery

### Automated Backups

```bash
# Daily backup (add to cron)
0 2 * * * cd /path/to/project && make backup
```

### Manual Backup

```bash
# Create backup
make backup

# Restore from backup  
make restore BACKUP_DIR=backups/20231215_020000
```

### Backup Contents

1. **Redis Data**: RDB snapshots + AOF logs
2. **PostgreSQL**: Full database dumps
3. **Application Logs**: Last 30 days
4. **Configuration**: Environment files

## Performance Tuning

### Redis Optimization

```conf
# redis.conf optimizations
maxmemory 2gb
maxmemory-policy allkeys-lru
tcp-keepalive 60
timeout 0
```

### API Optimization

```yaml
# Gunicorn settings
workers: 4
worker_class: uvicorn.workers.UvicornWorker
worker_connections: 1000
keepalive: 2
```

### Database Optimization

```yaml
# PostgreSQL settings  
shared_preload_libraries: 'pg_stat_statements'
work_mem: 256MB
effective_cache_size: 2GB
```

## Troubleshooting

### Common Issues

**Service won't start:**
```bash
# Check logs
make logs-api

# Check health
make health

# Restart service
docker-compose restart api
```

**High memory usage:**
```bash
# Check resource usage
docker stats

# Scale down if needed
make scale-api REPLICAS=2
```

**Redis connection errors:**
```bash
# Check Redis status
make shell-redis
> ping
PONG

# Clear cache if needed
> FLUSHALL
```

### Debug Mode

Enable debug logging:
```yaml
environment:
  - LOG_LEVEL=DEBUG
  - DEBUG=true
```

## Monitoring and Alerting

### Key Metrics

1. **Application Metrics**
   - Request rate and latency
   - Error rates by endpoint
   - Queue depths and processing time

2. **Infrastructure Metrics**
   - CPU/Memory/Disk usage
   - Network I/O
   - Container health

3. **Business Metrics**
   - Event processing rates
   - Detection accuracy
   - User activity

### Alert Rules

Critical alerts are configured for:
- High error rates (>5%)
- High latency (P99 > 2s)
- Service downtime
- Queue backlog (>1000 items)
- Resource exhaustion (>90%)

### Dashboards

Pre-configured Grafana dashboards:
1. **System Overview** - High-level metrics
2. **API Performance** - Request metrics
3. **Processing Pipeline** - ML workflow metrics
4. **Infrastructure** - System resources
5. **Business KPIs** - Detection rates

## Deployment Checklist

### Pre-deployment

- [ ] Environment variables configured
- [ ] SSL certificates in place
- [ ] Secrets created
- [ ] Backup strategy implemented
- [ ] Monitoring configured
- [ ] Load testing completed

### Deployment

- [ ] Build and test images
- [ ] Deploy to staging
- [ ] Run integration tests
- [ ] Deploy to production
- [ ] Verify health checks
- [ ] Monitor for issues

### Post-deployment

- [ ] Verify all services running
- [ ] Check metrics and logs
- [ ] Test critical workflows
- [ ] Update documentation
- [ ] Notify stakeholders

## Support

For deployment issues or questions:
- Create GitHub issue with deployment logs
- Check troubleshooting section above
- Contact DevOps team via Slack #insane-ai-ops

## Changelog

### v2.0.0 (Current)
- Added session-based processing
- Implemented autothrottle
- Enhanced security features
- Comprehensive monitoring

### v1.0.0 
- Initial Docker deployment
- Basic monitoring setup
- Development environment
