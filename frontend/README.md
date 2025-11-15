# Jamey 3.0 Frontend

React frontend for Jamey 3.0 - General & Guardian system. This frontend provides a UI for the Conscience Engine and Memory System.

## Quick Start

### 1. Install Dependencies

```bash
npm install
```

### 2. Configure Environment

Copy the example environment file and configure it:

```bash
cp .env.example .env
```

Edit `.env` and set:
- `VITE_API_URL` - Backend API URL (default: `http://localhost:3000`)
- `VITE_API_KEY` - Your API key (optional, if backend requires authentication)

### 3. Create API Key (If Needed)

If your backend requires API key authentication, create one:

**Option 1: Using Backend Script**
```bash
# From project root
./scripts/create-api-key.sh frontend-key 60
```

**Option 2: Programmatically**
```rust
// In backend code
let key_manager = ApiKeyManager::new(pool);
let key = key_manager.create_key("frontend-key", None, Some(60)).await?;
```

Then add the key to `frontend/.env`:
```
VITE_API_KEY=jamey_your-key-here
```

### 4. Start Development Server

```bash
npm run dev
```

The frontend will be available at `http://localhost:5173` (or the port Vite assigns).

### 5. Verify Connection

1. Make sure the backend is running on `http://localhost:3000`
2. Open the frontend in your browser
3. Try evaluating an action - you should see the moral score

## Configuration

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `VITE_API_URL` | No | `http://localhost:3000` | Backend API URL |
| `VITE_API_KEY` | No | - | API key for authentication |

### API Key Authentication

The frontend supports API key authentication via the `x-api-key` header. If `VITE_API_KEY` is set, it will be automatically included in all requests.

**Note**: If your backend doesn't require authentication, you can leave `VITE_API_KEY` empty.

## Available Scripts

- `npm run dev` - Start development server
- `npm run build` - Build for production
- `npm run preview` - Preview production build
- `npm run lint` - Run ESLint

## Integration with Backend

This frontend connects to the Jamey 3.0 backend API. See:
- [Frontend Integration Guide](../docs/FRONTEND_INTEGRATION.md) - Universal guide for any frontend
- [API Reference](../docs/API_REFERENCE.md) - Complete API documentation
- [Quick Start Guide](../docs/FRONTEND_QUICK_START.md) - 5-minute setup

## Project Structure

```
frontend/
├── src/
│   ├── api/
│   │   └── client.ts          # API client with authentication
│   ├── App.tsx                # Main application component
│   └── main.tsx               # React entry point
├── .env.example               # Environment template
└── vite.config.ts             # Vite configuration with proxy
```

## Troubleshooting

### Connection Issues

- Verify backend is running: `curl http://localhost:3000/health`
- Check `VITE_API_URL` matches your backend URL
- Check browser console for CORS errors

### Authentication Errors

- Verify `VITE_API_KEY` is set correctly in `.env`
- Check API key is valid in backend database
- Ensure backend allows the API key (not revoked/expired)

### CORS Errors

- Backend CORS is configured to allow all origins in development
- For production, configure `ALLOWED_ORIGINS` in backend `.env`

## Building for Production

```bash
npm run build
```

The built files will be in `dist/` directory. Deploy these files to your web server.

**Important**: Set environment variables in your production environment or build system, as Vite embeds them at build time.

---

This template provides a minimal setup to get React working in Vite with HMR and some ESLint rules.

Currently, two official plugins are available:

- [@vitejs/plugin-react](https://github.com/vitejs/vite-plugin-react/blob/main/packages/plugin-react) uses [Babel](https://babeljs.io/) (or [oxc](https://oxc.rs) when used in [rolldown-vite](https://vite.dev/guide/rolldown)) for Fast Refresh
- [@vitejs/plugin-react-swc](https://github.com/vitejs/vite-plugin-react/blob/main/packages/plugin-react-swc) uses [SWC](https://swc.rs/) for Fast Refresh

## React Compiler

The React Compiler is not enabled on this template because of its impact on dev & build performances. To add it, see [this documentation](https://react.dev/learn/react-compiler/installation).

## Expanding the ESLint configuration

If you are developing a production application, we recommend updating the configuration to enable type-aware lint rules:

```js
export default defineConfig([
  globalIgnores(['dist']),
  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      // Other configs...

      // Remove tseslint.configs.recommended and replace with this
      tseslint.configs.recommendedTypeChecked,
      // Alternatively, use this for stricter rules
      tseslint.configs.strictTypeChecked,
      // Optionally, add this for stylistic rules
      tseslint.configs.stylisticTypeChecked,

      // Other configs...
    ],
    languageOptions: {
      parserOptions: {
        project: ['./tsconfig.node.json', './tsconfig.app.json'],
        tsconfigRootDir: import.meta.dirname,
      },
      // other options...
    },
  },
])
```

You can also install [eslint-plugin-react-x](https://github.com/Rel1cx/eslint-react/tree/main/packages/plugins/eslint-plugin-react-x) and [eslint-plugin-react-dom](https://github.com/Rel1cx/eslint-react/tree/main/packages/plugins/eslint-plugin-react-dom) for React-specific lint rules:

```js
// eslint.config.js
import reactX from 'eslint-plugin-react-x'
import reactDom from 'eslint-plugin-react-dom'

export default defineConfig([
  globalIgnores(['dist']),
  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      // Other configs...
      // Enable lint rules for React
      reactX.configs['recommended-typescript'],
      // Enable lint rules for React DOM
      reactDom.configs.recommended,
    ],
    languageOptions: {
      parserOptions: {
        project: ['./tsconfig.node.json', './tsconfig.app.json'],
        tsconfigRootDir: import.meta.dirname,
      },
      // other options...
    },
  },
])
```
