# Rust Coding Standards
- Use `#[derive(Debug, Clone)]` for most structs
- Prefer `Arc<Mutex<T>>` or `Arc<DashMap>` for shared state
- Use `anyhow::Result` for application-level errors
- Implement proper error conversion with `?` operator
- Use `tracing::info!` instead of `println!` for logs
- Structure async functions with clear .await points
- Follow Rust API guidelines for public interfaces
