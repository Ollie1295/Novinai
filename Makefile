# Insane AI Security - Deployment Makefile
# Comprehensive Docker operations and deployment automation

.PHONY: help build up down logs clean test deploy prod-deploy dev-deploy

# Default target
help: ## Show this help message
	@echo "Insane AI Security - Docker Deployment Commands"
	@echo "================================================"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# Development commands
dev: ## Start development environment
	@echo "Starting development environment..."
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml up -d
	@echo "Services starting... Use 'make logs' to view output"
	@echo "API: http://localhost:8000"
	@echo "Grafana: http://localhost:3000 (admin/admin)"
	@echo "Redis Commander: http://localhost:8081"

dev-build: ## Build and start development environment
	@echo "Building and starting development environment..."
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml up --build -d

dev-logs: ## View development logs
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml logs -f

dev-down: ## Stop development environment
	@echo "Stopping development environment..."
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml down

# Production commands
prod: ## Start production environment
	@echo "Starting production environment..."
	@make check-secrets
	docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
	@echo "Production services starting..."
	@echo "API: https://api.insane.ai"
	@echo "Grafana: https://grafana.insane.ai"

prod-build: ## Build and start production environment
	@echo "Building production environment..."
	@make check-secrets
	docker-compose -f docker-compose.yml -f docker-compose.prod.yml up --build -d

prod-deploy: ## Deploy to production with zero downtime
	@echo "Deploying to production..."
	@make check-secrets
	@make backup
	docker-compose -f docker-compose.yml -f docker-compose.prod.yml pull
	docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d --no-deps --build api
	@echo "Waiting for health checks..."
	@sleep 30
	docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d --no-deps --build scheduler deep-worker
	@echo "Production deployment complete"

prod-logs: ## View production logs
	docker-compose -f docker-compose.yml -f docker-compose.prod.yml logs -f

prod-down: ## Stop production environment
	@echo "Stopping production environment..."
	docker-compose -f docker-compose.yml -f docker-compose.prod.yml down

# Build commands
build: ## Build all services
	@echo "Building all services..."
	docker-compose build --parallel

build-api: ## Build API service only
	docker-compose build api

build-scheduler: ## Build scheduler service only
	docker-compose build scheduler

build-worker: ## Build deep worker service only
	docker-compose build deep-worker

# Utility commands
up: ## Start all services (basic)
	docker-compose up -d

down: ## Stop all services
	docker-compose down

logs: ## View all service logs
	docker-compose logs -f

logs-api: ## View API logs only
	docker-compose logs -f api

logs-scheduler: ## View scheduler logs only
	docker-compose logs -f scheduler

logs-worker: ## View worker logs only
	docker-compose logs -f deep-worker

# Monitoring and health
status: ## Show service status
	@echo "Service Status:"
	@echo "==============="
	docker-compose ps
	@echo
	@echo "Health Checks:"
	@echo "============="
	@curl -s http://localhost:8000/health | jq . || echo "API not responding"
	@curl -s http://localhost:9090/-/healthy || echo "Prometheus not responding"
	@curl -s http://localhost:3000/api/health | jq . || echo "Grafana not responding"

health: ## Check service health
	@echo "Checking service health..."
	@docker-compose exec api python -c "import requests; print('API:', requests.get('http://localhost:8000/health').status_code)" 2>/dev/null || echo "API: Not running"
	@docker-compose exec redis redis-cli ping 2>/dev/null || echo "Redis: Not running"

# Database and data operations
backup: ## Backup production data
	@echo "Creating backup..."
	@mkdir -p backups/$(shell date +%Y%m%d_%H%M%S)
	docker-compose exec redis redis-cli --rdb /data/backup.rdb
	docker cp insane-ai-redis:/data/backup.rdb backups/$(shell date +%Y%m%d_%H%M%S)/redis.rdb
	docker-compose exec postgres pg_dump -U insane_ai insane_ai_prod > backups/$(shell date +%Y%m%d_%H%M%S)/postgres.sql 2>/dev/null || echo "PostgreSQL backup skipped (not running)"
	@echo "Backup completed in backups/$(shell date +%Y%m%d_%H%M%S)/"

restore: ## Restore from backup (specify BACKUP_DIR=...)
	@if [ -z "$(BACKUP_DIR)" ]; then echo "Usage: make restore BACKUP_DIR=backups/YYYYMMDD_HHMMSS"; exit 1; fi
	@echo "Restoring from $(BACKUP_DIR)..."
	docker cp $(BACKUP_DIR)/redis.rdb insane-ai-redis:/data/restore.rdb
	docker-compose exec redis redis-cli --rdb /data/restore.rdb
	@echo "Restore completed"

# Testing
test: ## Run all tests
	@echo "Running tests..."
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml exec api python -m pytest tests/ -v

test-load: ## Run load tests
	@echo "Running load tests..."
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml run --rm load-tester

test-security: ## Run security tests
	@echo "Running security tests..."
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml exec api python -m pytest tests/security/ -v

# Secrets management
setup-secrets: ## Setup production secrets
	@echo "Setting up production secrets..."
	@echo "Please provide the following secrets:"
	@read -s -p "JWT Secret: " jwt_secret; echo $$jwt_secret | docker secret create insane_ai_jwt_secret -
	@read -s -p "API Key: " api_key; echo $$api_key | docker secret create insane_ai_api_key -
	@read -s -p "Grafana Password: " grafana_password; echo $$grafana_password | docker secret create insane_ai_grafana_password -
	@echo "Secrets created successfully"

check-secrets: ## Check if production secrets exist
	@echo "Checking production secrets..."
	@docker secret inspect insane_ai_jwt_secret >/dev/null 2>&1 || (echo "Error: JWT secret not found. Run 'make setup-secrets'" && exit 1)
	@docker secret inspect insane_ai_api_key >/dev/null 2>&1 || (echo "Error: API key not found. Run 'make setup-secrets'" && exit 1)
	@echo "Secrets check passed"

# Cleanup commands
clean: ## Clean up containers and images
	@echo "Cleaning up..."
	docker-compose down --rmi all --volumes --remove-orphans
	docker system prune -f

clean-volumes: ## Remove all volumes (DANGEROUS)
	@echo "WARNING: This will remove all data volumes!"
	@read -p "Are you sure? (y/N) " -n 1 -r; echo; if [[ $$REPLY =~ ^[Yy]$$ ]]; then docker volume prune -f; fi

clean-all: ## Clean everything (DANGEROUS)
	@echo "WARNING: This will remove all containers, images, and volumes!"
	@read -p "Are you sure? (y/N) " -n 1 -r; echo; if [[ $$REPLY =~ ^[Yy]$$ ]]; then make clean && docker system prune -a -f --volumes; fi

# Development utilities
shell-api: ## Shell into API container
	docker-compose exec api /bin/bash

shell-redis: ## Shell into Redis container
	docker-compose exec redis redis-cli

shell-scheduler: ## Shell into scheduler container
	docker-compose exec scheduler /bin/bash

shell-worker: ## Shell into worker container
	docker-compose exec deep-worker /bin/bash

# Scaling commands
scale-api: ## Scale API service (usage: make scale-api REPLICAS=3)
	@if [ -z "$(REPLICAS)" ]; then echo "Usage: make scale-api REPLICAS=3"; exit 1; fi
	docker-compose up -d --scale api=$(REPLICAS)

scale-worker: ## Scale worker service (usage: make scale-worker REPLICAS=5)
	@if [ -z "$(REPLICAS)" ]; then echo "Usage: make scale-worker REPLICAS=5"; exit 1; fi
	docker-compose up -d --scale deep-worker=$(REPLICAS)

# Monitoring commands
metrics: ## View metrics
	@echo "Opening metrics dashboards..."
	@echo "Prometheus: http://localhost:9090"
	@echo "Grafana: http://localhost:3000"
	@echo "Redis Metrics: http://localhost:9121/metrics"

alerts: ## Check current alerts
	@echo "Current Prometheus alerts:"
	@curl -s http://localhost:9090/api/v1/alerts | jq '.data.alerts[] | {alertname: .labels.alertname, state: .state}' 2>/dev/null || echo "Prometheus not accessible"

# Update commands
update: ## Update all service images
	@echo "Updating service images..."
	docker-compose pull
	docker-compose up -d

# Initialization
init: ## Initialize the project for first run
	@echo "Initializing Insane AI Security..."
	@mkdir -p logs data tmp ssl/certs ssl/private monitoring/grafana/dashboards monitoring/grafana/datasources
	@echo "Project initialized. Run 'make dev' to start development environment."

# CI/CD commands
ci-test: ## Run CI tests
	@echo "Running CI test suite..."
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml run --rm api python -m pytest tests/ --junitxml=test-results.xml --cov=. --cov-report=xml

ci-security-scan: ## Run security scans
	@echo "Running security scans..."
	docker run --rm -v $(PWD):/src returntocorp/semgrep --config=auto /src

ci-build-test: ## Build and test (for CI)
	@echo "CI: Building and testing..."
	@make build
	@make ci-test
	@make ci-security-scan

# Documentation
docs: ## Generate documentation
	@echo "Generating documentation..."
	@echo "API Documentation: http://localhost:8000/docs"
	@echo "Metrics: http://localhost:9090"
	@echo "Dashboards: http://localhost:3000"
