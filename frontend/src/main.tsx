import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
<<<<<<< HEAD
=======
import { ErrorBoundary } from './components/ErrorBoundary'
>>>>>>> origin/main
import './index.css'
import App from './App.tsx'

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
<<<<<<< HEAD
      retry: 1,
=======
      retry: (failureCount, error: Error & { response?: { status?: number } }) => {
        // Don't retry on 4xx errors (client errors)
        if (error?.response?.status && error.response.status >= 400 && error.response.status < 500) {
          return false;
        }
        // Retry up to 3 times for network/server errors
        return failureCount < 3;
      },
      retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
      staleTime: 5 * 60 * 1000, // 5 minutes
    },
    mutations: {
      retry: false, // Don't retry mutations
>>>>>>> origin/main
    },
  },
})

createRoot(document.getElementById('root')!).render(
  <StrictMode>
<<<<<<< HEAD
    <QueryClientProvider client={queryClient}>
      <App />
    </QueryClientProvider>
=======
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <App />
      </QueryClientProvider>
    </ErrorBoundary>
>>>>>>> origin/main
  </StrictMode>,
)
