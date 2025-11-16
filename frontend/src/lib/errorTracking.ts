/**
 * Error Tracking Service
 * 
 * Provides centralized error tracking using Sentry (or console fallback in development).
 * Replaces console.error calls with proper error tracking.
 */

interface ErrorTrackingConfig {
  dsn?: string;
  environment?: string;
  enabled: boolean;
  sampleRate?: number;
}

class ErrorTrackingService {
  private config: ErrorTrackingConfig;
  private initialized = false;

  constructor(config: ErrorTrackingConfig) {
    this.config = config;
  }

  /**
   * Initialize error tracking
   */
  async init(): Promise<void> {
    if (this.initialized) return;

    // Only initialize in production or if explicitly enabled
    if (!this.config.enabled) {
      console.log('[ErrorTracking] Disabled - using console fallback');
      this.initialized = true;
      return;
    }

    // Try to load Sentry if DSN is provided
    if (this.config.dsn) {
      try {
        // Dynamic import to avoid bundling Sentry in dev if not needed
        const Sentry = await import('@sentry/react');
        
        Sentry.init({
          dsn: this.config.dsn,
          environment: this.config.environment || 'production',
          integrations: [
            Sentry.browserTracingIntegration(),
            Sentry.replayIntegration({
              maskAllText: true,
              blockAllMedia: true,
            }),
          ],
          tracesSampleRate: this.config.sampleRate || 0.1,
          replaysSessionSampleRate: 0.1,
          replaysOnErrorSampleRate: 1.0,
          beforeSend(event) {
            // Sanitize sensitive data
            if (event.request?.headers) {
              // Remove API keys and tokens
              const sanitized = { ...event.request.headers };
              if (sanitized['x-api-key']) {
                sanitized['x-api-key'] = '[REDACTED]';
              }
              if (sanitized['authorization']) {
                sanitized['authorization'] = '[REDACTED]';
              }
              event.request.headers = sanitized;
            }
            return event;
          },
        });

        this.initialized = true;
        console.log('[ErrorTracking] Sentry initialized');
      } catch (error) {
        console.warn('[ErrorTracking] Failed to initialize Sentry:', error);
        this.initialized = true; // Mark as initialized to prevent retries
      }
    } else {
      console.log('[ErrorTracking] No DSN provided - using console fallback');
      this.initialized = true;
    }
  }

  /**
   * Capture an error
   */
  captureError(error: Error, context?: Record<string, unknown>): void {
    if (!this.initialized) {
      this.init().catch(() => {});
    }

    // Log to console in development
    if (import.meta.env.DEV) {
      console.error('[ErrorTracking] Error captured:', error, context);
      return;
    }

    // Try to use Sentry if available
    if (this.config.enabled && this.config.dsn) {
      try {
        // Dynamic import - Sentry may not be loaded
        import('@sentry/react').then((Sentry) => {
          Sentry.captureException(error, {
            contexts: {
              custom: context || {},
            },
          });
        }).catch(() => {
          // Fallback to console if Sentry not available
          console.error('[ErrorTracking] Error:', error, context);
        });
      } catch {
        console.error('[ErrorTracking] Error:', error, context);
      }
    } else {
      console.error('[ErrorTracking] Error:', error, context);
    }
  }

  /**
   * Capture a message
   */
  captureMessage(message: string, level: 'info' | 'warning' | 'error' = 'info', context?: Record<string, unknown>): void {
    if (!this.initialized) {
      this.init().catch(() => {});
    }

    // Log to console in development
    if (import.meta.env.DEV) {
      console[level === 'error' ? 'error' : level === 'warning' ? 'warn' : 'log'](
        `[ErrorTracking] ${level}:`,
        message,
        context
      );
      return;
    }

    // Try to use Sentry if available
    if (this.config.enabled && this.config.dsn) {
      try {
        import('@sentry/react').then((Sentry) => {
          Sentry.captureMessage(message, {
            level: level === 'error' ? 'error' : level === 'warning' ? 'warning' : 'info',
            contexts: {
              custom: context || {},
            },
          });
        }).catch(() => {
          console[level === 'error' ? 'error' : level === 'warning' ? 'warn' : 'log'](
            `[ErrorTracking] ${level}:`,
            message,
            context
          );
        });
      } catch {
        console[level === 'error' ? 'error' : level === 'warning' ? 'warn' : 'log'](
          `[ErrorTracking] ${level}:`,
          message,
          context
        );
      }
    } else {
      console[level === 'error' ? 'error' : level === 'warning' ? 'warn' : 'log'](
        `[ErrorTracking] ${level}:`,
        message,
        context
      );
    }
  }

  /**
   * Set user context
   */
  setUser(user: { id?: string; username?: string; email?: string } | null): void {
    if (!this.initialized) {
      this.init().catch(() => {});
    }

    if (this.config.enabled && this.config.dsn) {
      try {
        import('@sentry/react').then((Sentry) => {
          Sentry.setUser(user);
        }).catch(() => {
          // Ignore if Sentry not available
        });
      } catch {
        // Ignore
      }
    }
  }

  /**
   * Add breadcrumb
   */
  addBreadcrumb(message: string, category?: string, level: 'info' | 'warning' | 'error' = 'info'): void {
    if (!this.initialized) {
      this.init().catch(() => {});
    }

    if (this.config.enabled && this.config.dsn) {
      try {
        import('@sentry/react').then((Sentry) => {
          Sentry.addBreadcrumb({
            message,
            category: category || 'default',
            level: level === 'error' ? 'error' : level === 'warning' ? 'warning' : 'info',
          });
        }).catch(() => {
          // Ignore if Sentry not available
        });
      } catch {
        // Ignore
      }
    }
  }
}

// Create singleton instance
const errorTracking = new ErrorTrackingService({
  dsn: import.meta.env.VITE_SENTRY_DSN,
  environment: import.meta.env.MODE || 'development',
  enabled: import.meta.env.PROD && !!import.meta.env.VITE_SENTRY_DSN,
  sampleRate: import.meta.env.VITE_SENTRY_SAMPLE_RATE 
    ? parseFloat(import.meta.env.VITE_SENTRY_SAMPLE_RATE) 
    : 0.1,
});

// Initialize on module load
errorTracking.init().catch(() => {});

export default errorTracking;

