# JAMEY 3.0 Architecture

## Core Components
1. **Conscience Engine** - Moral rule evaluation system
2. **5-Layer Memory** - Hierarchical memory storage
3. **REST API** - Axum-based web interface
4. **React Frontend** - Real-time interaction interface

## Data Flow
1. Action → Conscience Evaluation → Moral Score
2. Action + Score → Short-term Memory Storage
3. Memory layers can query and cross-reference
4. Frontend displays real-time evaluations

## Technology Choices
- **Rust**: Performance and safety for core AI logic
- **Tantivy**: Fast, embedded search for memory layers
- **Axum**: Modern async web framework
- **React 18**: Latest React with concurrent features
- **TanStack Query**: Server state management
