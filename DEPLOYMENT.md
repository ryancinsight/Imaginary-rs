# Deployment Guide - Imaginary-rs

This guide covers deploying Imaginary-rs in various environments from development to production.

## Quick Start

### Docker (Development)
```bash
# Build and run with Docker
docker build -t imaginary-rs .
docker run -p 8080:8080 imaginary-rs

# Or use Docker Compose
docker-compose up
```

### Docker Compose (Development with Monitoring)
```bash
# Start with monitoring stack
docker-compose --profile monitoring up

# Access services:
# - Application: http://localhost:8080
# - Prometheus: http://localhost:9090
# - Grafana: http://localhost:3000 (admin/admin)
```

## Production Deployment

### Prerequisites
- Docker and Docker Compose or Kubernetes cluster
- SSL certificates (for production)
- Secrets management system

### Docker Swarm (Production)
```bash
# Create secrets
echo "your-secure-api-key" | docker secret create api_key -
echo "your-secure-salt" | docker secret create api_salt -

# Deploy stack
docker stack deploy -c docker-compose.prod.yml imaginary-rs
```

### Kubernetes (Production)

#### 1. Create Namespace and Secrets
```bash
# Apply namespace
kubectl apply -f k8s/namespace.yaml

# Update secrets with your actual values
kubectl apply -f k8s/secret.yaml

# Apply configuration
kubectl apply -f k8s/configmap.yaml
```

#### 2. Deploy Application
```bash
# Deploy application
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml

# Setup ingress (update domain in ingress.yaml first)
kubectl apply -f k8s/ingress.yaml

# Enable autoscaling
kubectl apply -f k8s/hpa.yaml
```

#### 3. Verify Deployment
```bash
# Check pods
kubectl get pods -n imaginary-rs

# Check services
kubectl get svc -n imaginary-rs

# Check ingress
kubectl get ingress -n imaginary-rs

# View logs
kubectl logs -f deployment/imaginary-rs -n imaginary-rs
```

## Configuration

### Environment Variables
- `RUST_LOG`: Log level (trace, debug, info, warn, error)
- `IMAGINARY_ALLOW_INSECURE`: Allow insecure configuration (0/1)
- `IMAGINARY_API_KEY`: API key for authentication
- `IMAGINARY_SALT`: Salt for signature generation

### Configuration File
Configuration can be provided via TOML file:
```toml
[server]
host = "0.0.0.0"
port = 8080
read_timeout = 30
write_timeout = 30
max_body_size = 10485760

[security]
allowed_origins = ["*"]

[storage]
temp_dir = "/tmp"
max_cache_size = 1073741824
```

## Health Checks

The application provides several health endpoints:

- `/health` - Basic health check
- `/ready` - Readiness check with system validation
- `/metrics` - Prometheus-compatible metrics

### Docker Health Check
```bash
# Manual health check
docker run --rm imaginary-rs --health-check
```

### Kubernetes Health Checks
The deployment includes:
- **Liveness Probe**: `/health` endpoint
- **Readiness Probe**: `/ready` endpoint

## Monitoring and Observability

### Metrics
Prometheus-compatible metrics are available at `/metrics`:
- Request counts
- Error rates
- Response times
- Memory usage
- Uptime

### Logging
Structured JSON logging with configurable levels:
```bash
# Set log level
export RUST_LOG=info
```

### Alerting
Example Prometheus alerting rules:
```yaml
groups:
- name: imaginary-rs
  rules:
  - alert: ImageProcessingServiceDown
    expr: up{job="imaginary-rs"} == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Image processing service is down"
```

## Security

### Container Security
- Non-root user (uid 65532)
- Distroless base image
- Read-only root filesystem
- Minimal capabilities

### Network Security
- HTTPS/TLS termination at ingress
- Internal service communication
- CORS configuration
- API key authentication

### Secrets Management
- Kubernetes secrets for sensitive data
- Docker secrets in Swarm mode
- Environment variable injection

## Scaling

### Horizontal Scaling
- Kubernetes HPA based on CPU/memory
- Docker Swarm replicas
- Load balancing via ingress/nginx

### Vertical Scaling
Resource limits and requests:
```yaml
resources:
  requests:
    cpu: 100m
    memory: 128Mi
  limits:
    cpu: 500m
    memory: 512Mi
```

## Troubleshooting

### Common Issues

#### Health Check Failures
```bash
# Check application logs
kubectl logs -f deployment/imaginary-rs -n imaginary-rs

# Manual health check
kubectl exec -it deployment/imaginary-rs -n imaginary-rs -- /usr/local/bin/imaginary-rs --health-check
```

#### High Memory Usage
- Monitor `/metrics` endpoint
- Adjust `max_cache_size` configuration
- Scale horizontally with HPA

#### SSL/TLS Issues
- Verify certificate configuration
- Check ingress annotations
- Validate cert-manager setup

### Debug Commands
```bash
# Port forward for local testing
kubectl port-forward svc/imaginary-rs-service 8080:8080 -n imaginary-rs

# Execute shell in container
kubectl exec -it deployment/imaginary-rs -n imaginary-rs -- sh

# View configuration
kubectl get configmap imaginary-rs-config -n imaginary-rs -o yaml
```

## Performance Tuning

### Application Tuning
- Adjust `max_body_size` for large images
- Configure `read_timeout` and `write_timeout`
- Optimize `max_cache_size` based on available memory

### Container Tuning
- Set appropriate resource limits
- Use multi-stage builds for smaller images
- Enable compression middleware

### Infrastructure Tuning
- Use SSD storage for temporary files
- Configure appropriate node resources
- Implement CDN for static content

## Backup and Recovery

### Configuration Backup
```bash
# Backup Kubernetes resources
kubectl get all,configmap,secret -n imaginary-rs -o yaml > backup.yaml
```

### Disaster Recovery
- Store container images in multiple registries
- Backup configuration and secrets
- Document rollback procedures
- Test recovery procedures regularly

## Updates and Maintenance

### Rolling Updates
```bash
# Update image version
kubectl set image deployment/imaginary-rs imaginary-rs=ghcr.io/your-org/imaginary-rs:v1.1.0 -n imaginary-rs

# Check rollout status
kubectl rollout status deployment/imaginary-rs -n imaginary-rs

# Rollback if needed
kubectl rollout undo deployment/imaginary-rs -n imaginary-rs
```

### Maintenance Windows
- Schedule updates during low-traffic periods
- Use rolling updates to minimize downtime
- Monitor metrics during and after updates
- Have rollback plan ready

---

For additional support, see the [Operations Runbook](OPERATIONS.md) and [Troubleshooting Guide](TROUBLESHOOTING.md).
