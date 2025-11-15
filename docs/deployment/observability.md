# Jamey 3.0 Observability Guide

This document provides an overview of the monitoring and observability infrastructure for the Jamey 3.0 project. The entire stack is containerized and can be launched with a single command.

## Architecture

The observability stack consists of the following services:

- **Grafana**: The central hub for visualization. It provides dashboards for logs, metrics, and traces.
- **Loki**: A lightweight, horizontally-scalable log aggregation system.
- **Promtail**: The agent that collects logs from containers and ships them to Loki.
- **Prometheus**: A time-series database and monitoring system that scrapes metrics from the backend.
- **Jaeger**: An end-to-end distributed tracing system.
- **Alertmanager**: Handles alerts sent by Prometheus and routes them to notification channels.

## How to Launch

To launch the entire application and observability stack, run the following command from the root of the project:

```bash
docker-compose up -d
```

## Accessing the Tools

Once the services are running, you can access them at the following URLs:

- **Grafana**: [http://localhost:3001](http://localhost:3001)
  - **Login**: `admin` / `admin`
- **Jaeger UI**: [http://localhost:16686](http://localhost:16686)
- **Prometheus UI**: [http://localhost:9090](http://localhost:9090)
- **Alertmanager UI**: [http://localhost:9093](http://localhost:9093)

## Using the Observability Stack

### 1. Visualizing Metrics with Grafana

Grafana is pre-configured with a "Jamey 3.0 Overview" dashboard. To access it:

1.  Navigate to [http://localhost:3001](http://localhost:3001).
2.  Log in with the default credentials (`admin`/`admin`).
3.  On the left-hand menu, go to **Dashboards**.
4.  Click on the **Jamey 3.0 Overview** dashboard.

This dashboard provides a high-level view of key application metrics, including request latency, error rates, and throughput.

### 2. Exploring Logs with Loki

Grafana is also configured with a Loki data source, allowing you to query logs directly from the Grafana UI.

1.  In Grafana, go to the **Explore** view (compass icon on the left menu).
2.  At the top of the page, select the **Loki** data source from the dropdown.
3.  Use the **Log browser** to select labels to filter by (e.g., `{service="backend"}`).
4.  You can write LogQL queries to further refine your search.

### 3. Analyzing Traces with Jaeger

Jaeger allows you to trace the full lifecycle of a request as it moves through the system.

1.  Navigate to the **Jaeger UI** at [http://localhost:16686](http://localhost:16686).
2.  In the "Search" pane on the left, select `jamey-3-backend` from the **Service** dropdown.
3.  Click **Find Traces**.
4.  You will see a list of recent traces. Click on one to see a detailed flame graph of the request, including database queries and other operations.

### 4. Alerting with Prometheus and Alertmanager

Prometheus is configured to send alerts to Alertmanager if certain conditions are met (e.g., high error rates).

- You can view the status of alerts in the **Alertmanager UI** at [http://localhost:9093](http://localhost:9093).
- Alerting rules are defined in `prometheus/alert.rules.yml`.

By default, alerts are configured to be sent to a Slack channel. To enable this, set the `SLACK_API_URL` environment variable in `alertmanager/alertmanager.yml`.