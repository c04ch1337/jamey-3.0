# Stage 1: Planner - Determine dependency structure
FROM rust:1.78-slim-bullseye AS planner
WORKDIR /app
RUN cargo install cargo-chef --locked
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Builder - Build dependencies and application
FROM rust:1.78-slim-bullseye AS builder
WORKDIR /app
RUN cargo install cargo-chef --locked
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies first to leverage layer caching
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
# Build the application
RUN cargo build --release --bin jamey-3

# Stage 3: Final - Create a slim, secure final image
FROM debian:slim-bullseye AS final
WORKDIR /app
# Create a non-root user for security
RUN groupadd --system --gid 1001 appgroup && \
    useradd --system --uid 1001 --gid appgroup appuser
# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/jamey-3 /usr/local/bin/
# Copy migrations and configuration
COPY migrations ./migrations
COPY .env.example ./.env
RUN chown -R appuser:appgroup /app
USER appuser
# Set the entrypoint for the container
EXPOSE 8080
CMD ["/usr/local/bin/jamey-3"]