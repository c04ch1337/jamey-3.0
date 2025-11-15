# Port Reference Guide - Quick Reference

## Quick Port Reference

### Core Application
- **Backend API**: `3000` (configurable via `SERVER_PORT`)
- **Frontend**: `5173` (configurable via `FRONTEND_PORT`)

### Observability Stack
- **Grafana**: `3001` (configurable via `GRAFANA_PORT`)
- **Prometheus**: `9090` (configurable via `PROMETHEUS_PORT`)
- **Alertmanager**: `9093` (configurable via `ALERTMANAGER_PORT`)
- **Loki**: `3100` (configurable via `LOKI_PORT`)
- **Jaeger UI**: `16686` (configurable via `JAEGER_UI_PORT`)
- **Jaeger Agent**: `6831/udp` (configurable via `JAEGER_AGENT_PORT`)

### Optional Services
- **PostgreSQL**: `5432` (if enabled, configurable via `POSTGRES_PORT`)

## Environment Variables

```bash
# Core
export SERVER_PORT=3000
export FRONTEND_PORT=5173
export VITE_API_URL=http://localhost:3000

# Observability
export GRAFANA_PORT=3001
export PROMETHEUS_PORT=9090
export ALERTMANAGER_PORT=9093
export LOKI_PORT=3100
export JAEGER_UI_PORT=16686
export JAEGER_AGENT_PORT=6831
```

## Access URLs

### Development (Bare Metal)
- Backend: http://localhost:3000
- Frontend: http://localhost:5173

### Docker
- Backend: http://localhost:3000
- Frontend: http://localhost:5173
- Grafana: http://localhost:3001
- Prometheus: http://localhost:9090
- Alertmanager: http://localhost:9093
- Jaeger: http://localhost:16686

## Service Discovery (Docker Internal)

Within Docker network, services communicate using service names:
- `backend:3000` - Backend API
- `prometheus:9090` - Prometheus
- `loki:3100` - Loki
- `alertmanager:9093` - Alertmanager
- `jaeger:16686` - Jaeger UI
- `jaeger:6831` - Jaeger Agent (UDP)

