# Frontend & Agentic AI Configuration Review

## Frontend Configuration

### ✅ Currently Configured

**Frontend Environment Variable:**
- `VITE_API_URL` - Used in `frontend/src/api/client.ts`
  - Default: `http://localhost:3000`
  - Location: `frontend/src/api/client.ts:3`

### ⚠️ Missing Frontend Configuration

1. **Vite Proxy Target** (Hardcoded in `frontend/vite.config.ts:10`)
   ```typescript
   target: 'http://localhost:3000',  // Should be configurable
   ```
   
   **Should be configurable via:**
   - `VITE_PROXY_TARGET` (default: `http://localhost:3000`)
   - Or use `VITE_API_URL` for consistency

2. **Frontend Port** (Not configurable)
   - Vite dev server port is auto-assigned or uses default 5173
   - **Could add:** `VITE_PORT` (default: `5173`)

3. **Frontend Build Configuration**
   - Build output directory (default: `dist`)
   - **Could add:** `VITE_BUILD_DIR` (default: `dist`)

### Recommendation for Frontend

Add to `.env.example`:
```bash
# Frontend Configuration
VITE_API_URL=http://localhost:3000
# VITE_PROXY_TARGET=http://localhost:3000  # Optional, can use VITE_API_URL
```

Update `frontend/vite.config.ts`:
```typescript
const API_URL = process.env.VITE_API_URL || 'http://localhost:3000';

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      '/evaluate': {
        target: API_URL,
        changeOrigin: true,
      },
      '/rules': {
        target: API_URL,
        changeOrigin: true,
      },
    },
  },
})
```

---

## Agentic AI / Consciousness System Configuration

### ⚠️ Missing Configuration (All Hardcoded)

The consciousness system has several hardcoded thresholds and parameters that should be configurable:

#### 1. Global Workspace Settings

**Location:** `src/consciousness/global_workspace.rs:67`
```rust
competition_threshold: 0.7,  // Hardcoded
```

**Should be configurable via:**
- `CONSCIOUSNESS_COMPETITION_THRESHOLD` (default: `0.7`)
- Controls when information is broadcast through the workspace

**Location:** `src/consciousness/global_workspace.rs:55`
```rust
let (tx, rx) = mpsc::channel(100);  // Hardcoded channel size
```

**Should be configurable via:**
- `CONSCIOUSNESS_BROADCAST_CHANNEL_SIZE` (default: `100`)

#### 2. Integrated Information (Φ) Settings

**Location:** `src/consciousness/mod.rs:127`
```rust
self.metrics.read().await.phi_value >= 0.85  // Hardcoded consciousness threshold
```

**Should be configurable via:**
- `CONSCIOUSNESS_PHI_THRESHOLD` (default: `0.85`)
- Minimum Φ value to consider the system "conscious"

**Location:** `src/consciousness/integrated_info.rs:53`
```rust
epsilon: 1e-6,  // Hardcoded epsilon for calculations
```

**Should be configurable via:**
- `CONSCIOUSNESS_PHI_EPSILON` (default: `1e-6`)
- Minimum information difference to consider in Φ calculations

#### 3. Activity Level Factors

**Location:** `src/consciousness/global_workspace.rs:116-117`
```rust
let broadcast_factor = if state.current_broadcast.is_some() { 0.5 } else { 0.0 };
let competition_factor = (state.competition_level as f64 / 10.0).min(0.5);
```

**Should be configurable via:**
- `CONSCIOUSNESS_BROADCAST_FACTOR` (default: `0.5`)
- `CONSCIOUSNESS_COMPETITION_DIVISOR` (default: `10.0`)
- `CONSCIOUSNESS_COMPETITION_MAX_FACTOR` (default: `0.5`)

### Recommended Configuration Structure

Add to `.env.example`:

```bash
# Consciousness System Configuration
# Global Workspace
CONSCIOUSNESS_COMPETITION_THRESHOLD=0.7
CONSCIOUSNESS_BROADCAST_CHANNEL_SIZE=100

# Integrated Information (Φ)
CONSCIOUSNESS_PHI_THRESHOLD=0.85
CONSCIOUSNESS_PHI_EPSILON=0.000001

# Activity Level Calculation
CONSCIOUSNESS_BROADCAST_FACTOR=0.5
CONSCIOUSNESS_COMPETITION_DIVISOR=10.0
CONSCIOUSNESS_COMPETITION_MAX_FACTOR=0.5
```

### Implementation Recommendation

Create a `ConsciousnessConfig` struct similar to `SoulConfig`:

```rust
// In src/config/mod.rs or src/consciousness/config.rs
pub struct ConsciousnessConfig {
    pub competition_threshold: f64,
    pub broadcast_channel_size: usize,
    pub phi_threshold: f64,
    pub phi_epsilon: f64,
    pub broadcast_factor: f64,
    pub competition_divisor: f64,
    pub competition_max_factor: f64,
}

impl ConsciousnessConfig {
    pub fn from_env() -> Self {
        Self {
            competition_threshold: env::var("CONSCIOUSNESS_COMPETITION_THRESHOLD")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.7),
            broadcast_channel_size: env::var("CONSCIOUSNESS_BROADCAST_CHANNEL_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            phi_threshold: env::var("CONSCIOUSNESS_PHI_THRESHOLD")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.85),
            phi_epsilon: env::var("CONSCIOUSNESS_PHI_EPSILON")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1e-6),
            broadcast_factor: env::var("CONSCIOUSNESS_BROADCAST_FACTOR")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.5),
            competition_divisor: env::var("CONSCIOUSNESS_COMPETITION_DIVISOR")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10.0),
            competition_max_factor: env::var("CONSCIOUSNESS_COMPETITION_MAX_FACTOR")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.5),
        }
    }
}
```

---

## Summary

### Frontend Settings
- ✅ `VITE_API_URL` is used (but not in `.env.example`)
- ⚠️ Vite proxy target is hardcoded
- ⚠️ Frontend settings not documented in `.env.example`

### Agentic AI / Consciousness Settings
- ❌ **No configuration** - all thresholds are hardcoded
- ❌ **No environment variables** for consciousness system
- ❌ **Not documented** in `.env.example`

### Priority Actions

1. **High Priority:**
   - Add `VITE_API_URL` to `.env.example`
   - Update `vite.config.ts` to use `VITE_API_URL` for proxy

2. **Medium Priority:**
   - Create `ConsciousnessConfig` struct
   - Add consciousness settings to `.env.example`
   - Update consciousness modules to use config

3. **Low Priority:**
   - Add frontend port/build directory settings (if needed)

