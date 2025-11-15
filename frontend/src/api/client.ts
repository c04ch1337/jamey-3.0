import axios, { AxiosError } from 'axios';
import { z } from 'zod';
import errorTracking from '../lib/errorTracking';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';
const API_KEY = import.meta.env.VITE_API_KEY;
// Support both x-api-key and Authorization: Bearer formats
// Set VITE_API_KEY_FORMAT=bearer to use Authorization: Bearer header instead
const API_KEY_FORMAT = import.meta.env.VITE_API_KEY_FORMAT || 'x-api-key';

// Request timeout (30 seconds)
const REQUEST_TIMEOUT = 30000;

// Zod schemas for API response validation
export const EvaluateResponseSchema = z.object({
  score: z.number(),
  action: z.string(),
});

export const MoralRuleSchema = z.object({
  name: z.string(),
  description: z.string(),
  weight: z.number(),
});

// Input sanitization function
const sanitizeInput = (input: string): string => {
  return input
    .trim()
    .replace(/[<>]/g, '') // Remove potential HTML tags
    .replace(/javascript:/gi, '') // Remove javascript: protocol
    .replace(/on\w+=/gi, '') // Remove event handlers
    .slice(0, 1000); // Enforce max length
};

export const AddRuleRequestSchema = z.object({
  name: z.string().min(1, 'Rule name is required').max(100, 'Rule name too long').transform(sanitizeInput),
  description: z.string().min(1, 'Description is required').max(500, 'Description too long').transform(sanitizeInput),
  weight: z.number().min(0, 'Weight must be >= 0').max(100, 'Weight must be <= 100'),
});

export const apiClient = axios.create({
  baseURL: API_BASE_URL,
  timeout: REQUEST_TIMEOUT,
  headers: {
    'Content-Type': 'application/json',
    // Include API key if available (supports both x-api-key and Authorization: Bearer formats)
    ...(API_KEY && (API_KEY_FORMAT === 'bearer' 
      ? { 'Authorization': `Bearer ${API_KEY}` }
      : { 'x-api-key': API_KEY })),
  },
});

// Generates a mock traceparent header.
// In a real-world scenario, you would use OpenTelemetry JS to generate this.
const generateTraceParent = (): string => {
  const version = '00';
  const traceId = crypto.randomUUID().replace(/-/g, '');
  const spanId = crypto.randomUUID().replace(/-/g, '').substring(0, 16);
  const flags = '01'; // Sampled
  return `${version}-${traceId}-${spanId}-${flags}`;
};

// Request interceptor - can add auth tokens here if needed
apiClient.interceptors.request.use(
  (config) => {
    // Add auth token from localStorage if available (takes precedence over env API key)
    const token = localStorage.getItem('auth_token');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    // Add traceparent header for distributed tracing
    config.headers['traceparent'] = generateTraceParent();
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

// Response interceptor for centralized error handling
apiClient.interceptors.response.use(
  (response) => response,
  (error: AxiosError) => {
    // Handle common errors
    if (error.response) {
      // Server responded with error status
      const status = error.response.status;
      const data = error.response.data as Record<string, unknown>;
      
      if (status === 401) {
        // Unauthorized - clear token if exists
        localStorage.removeItem('auth_token');
        errorTracking.captureMessage('Unauthorized - authentication required', 'error', {
          status,
          path: error.config?.url,
        });
      } else if (status === 403) {
        errorTracking.captureMessage('Forbidden - insufficient permissions', 'error', {
          status,
          path: error.config?.url,
        });
      } else if (status >= 500) {
        errorTracking.captureError(
          new Error(`Server error: ${data?.message || error.message}`),
          { status, path: error.config?.url, data }
        );
      } else if (status === 404) {
        errorTracking.captureMessage('Resource not found', 'warning', {
          status,
          path: error.config?.url,
        });
      } else if (status === 429) {
        errorTracking.captureMessage('Rate limit exceeded', 'warning', {
          status,
          path: error.config?.url,
        });
      } else {
        errorTracking.captureError(
          new Error(`API Error (${status}): ${data?.message || error.message}`),
          { status, path: error.config?.url, data }
        );
      }
    } else if (error.request) {
      // Request made but no response received
      if (error.code === 'ECONNABORTED') {
        errorTracking.captureError(
          new Error('Request timeout - server took too long to respond'),
          { code: error.code, path: error.config?.url }
        );
      } else {
        errorTracking.captureError(
          new Error('Network error - unable to reach server'),
          { code: error.code, path: error.config?.url }
        );
      }
    } else {
      // Error setting up request
      errorTracking.captureError(
        new Error(`Request setup error: ${error.message}`),
        { path: error.config?.url }
      );
    }
    
    return Promise.reject(error);
  }
);

export interface EvaluateRequest {
  action: string;
}

export interface EvaluateResponse {
  score: number;
  action: string;
}

export interface MoralRule {
  name: string;
  description: string;
  weight: number;
}

export interface AddRuleRequest {
  name: string;
  description: string;
  weight: number;
}

// API functions with validation
export const evaluateAction = async (action: string): Promise<EvaluateResponse> => {
  // Validate input
  if (!action || action.trim().length === 0) {
    throw new Error('Action cannot be empty');
  }
  if (action.length > 1000) {
    throw new Error('Action too long (max 1000 characters)');
  }

  const response = await apiClient.post<EvaluateResponse>('/evaluate', { action });
  
  // Validate response
  return EvaluateResponseSchema.parse(response.data);
};

export const getRules = async (): Promise<MoralRule[]> => {
  const response = await apiClient.get<MoralRule[]>('/rules');
  
  // Validate response array
  const rules = z.array(MoralRuleSchema).parse(response.data);
  return rules;
};

export const addRule = async (rule: AddRuleRequest): Promise<void> => {
  // Validate input
  const validated = AddRuleRequestSchema.parse(rule);
  
  await apiClient.post('/rules', validated);
};
