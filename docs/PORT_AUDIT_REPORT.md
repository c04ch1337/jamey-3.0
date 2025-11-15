# Port Configuration Audit & Validation Report

## Executive Summary

This document provides a comprehensive audit of all ports used in the Jamey 3.0 application, ensuring consistency between Docker and bare metal deployments.

---

## Port Inventory

### Core Application Ports

| Service | Port | Docker Internal | Docker External | Bare Metal | Status |
|---------|------|----------------|-----------------|------------|--------|
| **Backend API** | 3000 | 3000 | ${SERVER_PORT:-3000} | 3000 | ✅ Consistent |
| **Frontend** | 5173 | 80 | ${FRONTEND_PORT:-5173} | 5173 | ✅ Consistent |

### Observability Stack Ports

| Service | Port | Docker Internal | Docker External | Bare Metal | Status |
|---------|------|----------------|-----------------|------------|--------|
| **Grafana** | 3000 | 3000 | ${GRAFANA_PORT:-3001} | N/A (Docker only) | ✅ Fixed (was 3000) |
| **Prometheus** | 9090 | 9090 | ${PROMETHEUS_PORT:-9090} | 9090 | ✅ Consistent |
| **Alertmanager** | 9093 | 9093 | ${ALERTMANAGER_PORT:-9093} | 9093 | ✅ Consistent |
| **Loki** | 3100 | 3100 | ${LOKI_PORT:-3100} | 3100 | ✅ Consistent |
| **Promtail** | 9080 | 9080 | N/A (internal only) | 9080 | ✅ Consistent |
| **Jaeger UI** | 16686 | 16686 | ${JAEGER_UI_PORT:-16686} | 16686 | ✅ Consistent |
| **Jaeger Agent** | 6831 | 6831/udp | ${JAEGER_AGENT_PORT:-6831}/udp | 6831/udp | ✅ Consistent |

### Optional Service Ports

| Service | Port | Docker External | Status | Notes |
|---------|------|----------------|--------|-------|
| **PostgreSQL** | 5432 | ${POSTGRES_PORT:-5432} | ⚠️ Optional | Commented out (app uses SQLite) |
| **Node Exporter** | 9100 | N/A | ❌ Missing | Referenced but not in docker-compose |
| **Consciousness Service** | 9090 | N/A | ❌ Missing | Referenced but doesn't exist |

---

## Issues Found & Fixed

### 🔴 CRITICAL Issues

#### 1. Prometheus Configuration - Wrong Backend Port
**File**: `prometheus/prometheus.yml`
**Issue**: References `backend:8080` but backend uses port 3000
**Impact**: Prometheus cannot scrape backend metrics
**Fix**: Changed to `backend:3000`

#### 2. Prometheus Configuration - Non-existent Services
**File**: `prometheus/prometheus.yml`
**Issue**: References `node-exporter:9100` and `consciousness:9090` which don't exist
**Impact**: Prometheus scrape failures, false errors
**Fix**: Removed non-existent service references

#### 3. Frontend Nginx Configuration - Wrong Backend Port
**File**: `frontend/nginx.conf`
**Issue**: References `backend:8080` but backend uses port 3000
**Impact**: Frontend cannot connect to backend in Docker
**Fix**: Changed to `backend:3000`

### 🟡 MEDIUM Issues

#### 4. Documentation - Wrong Grafana Port
**File**: `docs/deployment/observability.md`
**Issue**: Says Grafana is on port 3000, but it's actually 3001
**Impact**: Users can't access Grafana
**Fix**: Updated to port 3001

#### 5. Documentation - Wrong Backend Port
**File**: `docs/deployment/containerization.md`
**Issue**: Says backend is on port 8080, but it's 3000
**Impact**: Confusion for developers
**Fix**: Updated to port 3000

---

## Port Configuration Reference

### Environment Variables

```bash
# Core Application
SERVER_PORT=3000                    # Backend API port
FRONTEND_PORT=5173                  # Frontend exposed port
VITE_API_URL=http://localhost:3000 # Frontend API URL

# Observability
GRAFANA_PORT=3001                   # Grafana UI (avoids conflict with backend)
PROMETHEUS_PORT=9090                # Prometheus UI
ALERTMANAGER_PORT=9093              # Alertmanager UI
LOKI_PORT=3100                      # Loki API
JAEGER_UI_PORT=16686                # Jaeger UI
JAEGER_AGENT_PORT=6831              # Jaeger agent (UDP)

# Optional
POSTGRES_PORT=5432                  # PostgreSQL (if enabled)
```

### Docker Compose Port Mappings

```yaml
backend:     "${SERVER_PORT:-3000}:${SERVER_PORT:-3000}"
frontend:     "${FRONTEND_PORT:-5173}:80"
grafana:     "${GRAFANA_PORT:-3001}:3000"
prometheus:  "${PROMETHEUS_PORT:-9090}:9090"
alertmanager: "${ALERTMANAGER_PORT:-9093}:9093"
loki:        "${LOKI_PORT:-3100}:3100"
jaeger:      "${JAEGER_UI_PORT:-16686}:16686"
jaeger:      "${JAEGER_AGENT_PORT:-6831}:6831/udp"
```

### Bare Metal Default Ports

```bash
Backend:      3000
Frontend:     5173 (Vite dev server)
Grafana:      N/A (Docker only)
Prometheus:   9090 (if running locally)
Alertmanager: 9093 (if running locally)
Loki:         3100 (if running locally)
Jaeger:       16686 (if running locally)
```

---

## Consistency Matrix

### Backend Port (3000)
- ✅ `src/main.rs`: Uses `SERVER_PORT` env var, defaults to 3000
- ✅ `docker-compose.yml`: Maps `${SERVER_PORT:-3000}:${SERVER_PORT:-3000}`
- ✅ `frontend/src/api/client.ts`: Defaults to `http://localhost:3000`
- ✅ `frontend/vite.config.ts`: Defaults to `http://localhost:3000`
- ✅ `README.md`: Documents port 3000
- ❌ `prometheus/prometheus.yml`: Was `backend:8080` → **FIXED** to `backend:3000`
- ❌ `frontend/nginx.conf`: Was `backend:8080` → **FIXED** to `backend:3000`
- ❌ `docs/deployment/containerization.md`: Was 8080 → **FIXED** to 3000

### Frontend Port (5173)
- ✅ `docker-compose.yml`: Maps `${FRONTEND_PORT:-5173}:80`
- ✅ `frontend/vite.config.ts`: Vite dev server uses 5173
- ✅ `src/api/mod.rs`: CORS defaults include `http://localhost:5173`
- ✅ `README.md`: Documents port 5173

### Grafana Port (3001)
- ✅ `docker-compose.yml`: Maps `${GRAFANA_PORT:-3001}:3000`
- ❌ `docs/deployment/observability.md`: Was 3000 → **FIXED** to 3001

### Prometheus Port (9090)
- ✅ `docker-compose.yml`: Maps `${PROMETHEUS_PORT:-9090}:9090`
- ✅ `grafana/provisioning/datasources/all.yml`: `http://prometheus:9090`
- ✅ `prometheus/prometheus.yml`: Alertmanager at `alertmanager:9093`
- ✅ All documentation: Correctly references 9090

---

## Service Discovery Ports (Docker Internal)

These ports are used for inter-service communication within Docker:

| Service | Internal Port | Used By |
|---------|--------------|---------|
| backend | 3000 | frontend, prometheus |
| prometheus | 9090 | grafana |
| loki | 3100 | grafana, promtail |
| alertmanager | 9093 | prometheus |
| jaeger | 16686 | grafana |
| jaeger | 6831/udp | backend |

---

## Production Deployment Notes

### Port Conflicts to Avoid

1. **Backend (3000)** vs **Grafana (3001)**: ✅ Resolved - Grafana moved to 3001
2. **Frontend (5173)**: ✅ No conflicts - Standard Vite port
3. **All observability ports**: ✅ Standard ports, no conflicts

### Firewall Considerations

If deploying to production, ensure these ports are accessible:
- **Public**: 3000 (backend), 5173 (frontend), 3001 (Grafana - optional)
- **Internal Only**: 9090 (Prometheus), 9093 (Alertmanager), 3100 (Loki), 16686 (Jaeger)

### Reverse Proxy Configuration

If using nginx/traefik as reverse proxy:
```nginx
# Backend API
location /api/ {
    proxy_pass http://localhost:3000/;
}

# Frontend
location / {
    proxy_pass http://localhost:5173/;
}

# Grafana (optional)
location /grafana/ {
    proxy_pass http://localhost:3001/;
}
```

---

## Validation Checklist

- [x] Backend port consistent across all configs (3000)
- [x] Frontend port consistent across all configs (5173)
- [x] Grafana port documented correctly (3001)
- [x] Prometheus config references correct backend port
- [x] Frontend nginx config references correct backend port
- [x] All documentation updated with correct ports
- [x] Docker compose uses environment variables for flexibility
- [x] Bare metal defaults match Docker defaults
- [x] No port conflicts between services
- [x] Service discovery ports correct for Docker networking

---

## Summary

**Total Issues Found**: 5
**Critical Issues**: 3
**Medium Issues**: 2
**All Issues**: ✅ **FIXED**

All ports are now consistent between Docker and bare metal deployments. The configuration is production-ready with proper environment variable support for flexibility.

