# Containerization and CI/CD

This document provides an overview of the containerization and CI/CD pipeline for the Jamey 3.0 project.

## Development Environments

This project supports two primary development environments:

1.  **Bare-Metal**: The traditional setup, where the developer is responsible for installing and managing all dependencies (e.g., Rust, Node.js, PostgreSQL) on their local machine. This remains the primary, fully supported development path.
2.  **Containerized**: An alternative, Docker-based setup that provides a consistent, portable, and isolated development environment. This is recommended for developers who prefer to avoid manual dependency management.

### Bare-Metal Setup

For instructions on setting up a bare-metal development environment, please refer to the main `README.md` and the `scripts/setup.sh` file.

### Containerized Setup

The containerized setup uses Docker and `docker-compose` to orchestrate the entire application stack, including the backend, frontend, and database.

**Prerequisites:**

*   Docker
*   `docker-compose`

**To start the containerized environment, run the following command from the project root:**

```bash
docker-compose up
```

This will build the necessary Docker images and start the following services:

*   `backend`: The Rust backend, accessible at `http://localhost:3000`.
*   `frontend`: The React frontend, accessible at `http://localhost:5173` (or the port specified by `FRONTEND_PORT`).
*   `db`: A PostgreSQL database instance.

Live reloading is enabled for both the frontend and backend, so any changes to the source code will trigger an automatic rebuild and restart of the corresponding service.

## CI/CD Pipeline

The project includes a basic CI/CD pipeline using GitHub Actions, defined in `.github/workflows/ci.yml`. This pipeline is triggered on every push and pull request to the `main` branch and performs the following checks:

*   **Linting**: Ensures that the code adheres to the established style guidelines using `cargo fmt` and `cargo clippy`.
*   **Testing**: Runs the full test suite using `cargo test --workspace`.
*   **Building**: Verifies that the Docker images for both the frontend and backend can be built successfully.

This automated pipeline helps to maintain code quality, prevent regressions, and ensure that the application is always in a deployable state.