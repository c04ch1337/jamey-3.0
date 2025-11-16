# Jamey 3.0 - General & Guardian

<div align="center">

![Version](https://img.shields.io/badge/version-3.0.0-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![License](https://img.shields.io/badge/license-Eternal%20Hive-purple.svg)
![Status](https://img.shields.io/badge/status-active-success.svg)

[![Rust](https://img.shields.io/badge/Backend-Rust-orange?logo=rust)](https://www.rust-lang.org/)
[![SQLite](https://img.shields.io/badge/Database-SQLite-blue?logo=sqlite)](https://www.sqlite.org/)
[![MQTT](https://img.shields.io/badge/Protocol-MQTT-orange?logo=eclipsemosquitto)](https://mqtt.org/)

[![Axum](https://img.shields.io/badge/Web%20Framework-Axum-0.7-green)](https://github.com/tokio-rs/axum)
[![Tokio](https://img.shields.io/badge/Async-Tokio-1.0-blue)](https://tokio.rs/)

[![Conscience Engine](https://img.shields.io/badge/Feature-Conscience%20Engine-purple)](./src/conscience/)
[![Memory System](https://img.shields.io/badge/Feature-5--Layer%20Memory-blue)](./src/memory/)
[![Soul KB](https://img.shields.io/badge/Feature-Soul%20KB-pink)](./src/soul/)
[![MQTT Client](https://img.shields.io/badge/Feature-MQTT%20Client-orange)](./src/mqtt/)
[![Security](https://img.shields.io/badge/Security-A%2B%20Grade-green)](./docs/PHASE2_SECURITY_HARDENING.md)

[![API](https://img.shields.io/badge/API-REST%20API-green)](./docs/API_REFERENCE.md)
[![CLI](https://img.shields.io/badge/CLI-Interactive%20Chat-blue)](./src/bin/jamey-cli.rs)
[![Documentation](https://img.shields.io/badge/Docs-Comprehensive-blue)](./docs/)

[![Eternal Hive](https://img.shields.io/badge/Part%20of-Eternal%20Hive-purple)](https://github.com/TransformArmyAI/Eternal-Hive)
[![Transform Army AI](https://img.shields.io/badge/Transform-Army%20AI-black)](https://github.com/TransformArmyAI)

</div>

Jamey 3.0 is the digital mirror and guardian system, part of the Eternal Hive architecture. It features a Conscience Engine for moral evaluation, a 5-Layer Memory System for persistent knowledge storage, and comprehensive monitoring and observability capabilities.

## Core Features

- **Smart Caching System**: Efficient multi-layer caching with automatic metrics collection
- **Async Communication**: Advanced async patterns with bounded channels and backpressure
- **Context Management**: Sophisticated context tracking with relevance-based retrieval
- **Comprehensive Monitoring**: Full observability with Prometheus integration
- **Enterprise Security**: DDoS protection, threat detection, and automated incident response

## Architecture

- **Backend**: Rust with Tokio, Axum, SQLx
- **AI**: Conscience Engine with weighted moral rules
- **Memory**: 5-Layer Memory System (Short-term, Long-term, Working, Episodic, Semantic)
- **Database**: SQLite with SQLx
- **Real-time**: MQTT async client with authentication and pub/sub capabilities
- **Soul KB**: Emoji-based emotion tracking with trust scoring
- **Security**: Enterprise-grade security with A+ grade
- **Monitoring**: Prometheus integration with custom metrics and alerts

## Project Structure

```
jamey-3.0/
├── src/                    # Rust backend source
│   ├── main.rs            # Application entry point
│   ├── lib.rs             # Library root
│   ├── api/               # Axum API routes
│   ├── conscience/        # Conscience Engine
│   ├── memory/           # 5-Layer Memory System
│   ├── metrics/          # Metrics collection
│   ├── telemetry/        # OpenTelemetry integration
│   └── db/               # Database layer
├── prometheus/           # Prometheus configuration
│   ├── prometheus.yml    # Main config
│   └── rules/           # Alert rules
├── migrations/           # SQLx database migrations
├── data/                # Database and memory indices
└── docs/                # Comprehensive documentation
    ├── CACHING.md       # Caching system guide
    ├── ASYNC_PATTERNS.md # Async communication patterns
    ├── CONTEXT_MANAGEMENT.md # Context system guide
    ├── MONITORING.md    # Monitoring and metrics guide
    └── TROUBLESHOOTING.md # Troubleshooting guide
```

## Getting Started

### Prerequisites

- Rust (latest stable)
- SQLite
- Prometheus (optional, for monitoring)

### Quick Start

1. Clone the repository:
```bash
git clone https://github.com/your-org/jamey-3.0.git
cd jamey-3.0
```

2. Copy environment template:
```bash
cp .env.example .env
```

3. Build and run:
```bash
cargo build
cargo run
```

The server will start on `http://localhost:3000`

## Documentation

### Core Systems
- [Caching System](docs/CACHING.md)
- [Async Patterns](docs/ASYNC_PATTERNS.md)
- [Context Management](docs/CONTEXT_MANAGEMENT.md)
- [Monitoring Guide](docs/MONITORING.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)

### Configuration
- [Configuration Guide](docs/CONFIGURATION.md)
- [Environment Template](docs/env-template.md)
- [Configuration Schema](docs/configuration-schema.md)

### Architecture
- [System Architecture](docs/architecture.md)
- [MQTT Architecture](docs/mqtt_architecture.md)
- [Soul KB Design](docs/phase_4_6_architecture.md)

## Monitoring & Observability

### Metrics Collection
- HTTP request metrics
- Memory system performance
- Cache hit rates
- System resources
- Custom business metrics

### Alert Rules
- System availability
- Performance degradation
- Resource constraints
- Security incidents
- Business logic alerts

### Dashboards
- System overview
- Performance metrics
- Error tracking
- Resource utilization
- Business insights

## Development

### Running Tests
```bash
# Full test suite
./scripts/test-all.sh

# With coverage
./scripts/test-coverage.sh
```

### Production Build
```bash
cargo build --release
```

## Security Features

- DDoS protection
- Rate limiting
- CSRF protection
- Security headers
- Threat detection
- Incident response
- Secret rotation
- Input validation

## Performance Features

- Smart caching system
- Async communication
- Connection pooling
- Resource monitoring
- Performance alerts
- Automatic scaling
- Load balancing

## License

Part of the Eternal Hive project - Transform Army AI
