import { Component } from 'react';
import type { ReactNode } from 'react';
import errorTracking from '../lib/errorTracking';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error?: Error;
  errorInfo?: React.ErrorInfo;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    // Track error with error tracking service
    errorTracking.captureError(error, {
      componentStack: errorInfo.componentStack,
      errorBoundary: true,
    });
    this.setState({ errorInfo });
  }

  handleReload = () => {
    window.location.reload();
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          minHeight: '100vh',
          padding: '2rem',
          textAlign: 'center',
        }}>
          <h2 style={{ color: '#c33', marginBottom: '1rem' }}>
            Something went wrong
          </h2>
          <p style={{ color: '#666', marginBottom: '1rem' }}>
            {this.state.error?.message || 'An unexpected error occurred'}
          </p>
          {import.meta.env.DEV && this.state.errorInfo && (
            <details style={{ marginBottom: '1rem', textAlign: 'left', maxWidth: '600px' }}>
              <summary style={{ cursor: 'pointer', color: '#667eea' }}>
                Error Details (Development Only)
              </summary>
              <pre style={{
                background: '#f5f5f5',
                padding: '1rem',
                borderRadius: '4px',
                overflow: 'auto',
                fontSize: '0.875rem',
              }}>
                {this.state.error?.stack}
                {'\n\n'}
                {this.state.errorInfo.componentStack}
              </pre>
            </details>
          )}
          <button
            onClick={this.handleReload}
            style={{
              padding: '0.75rem 1.5rem',
              background: '#667eea',
              color: 'white',
              border: 'none',
              borderRadius: '8px',
              fontSize: '1rem',
              cursor: 'pointer',
            }}
          >
            Reload Page
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}

