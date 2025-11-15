import axios, { AxiosError } from 'axios';
import { z } from 'zod';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

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
    // Add auth token if available
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
        console.error('Unauthorized - authentication required');
      } else if (status === 403) {
        console.error('Forbidden - insufficient permissions');
      } else if (status >= 500) {
        console.error('Server error - please try again later');
      } else if (status === 404) {
        console.error('Resource not found');
      } else {
        console.error(`API Error (${status}):`, data?.message || error.message);
      }
    } else if (error.request) {
      // Request made but no response received
      if (error.code === 'ECONNABORTED') {
        console.error('Request timeout - server took too long to respond');
      } else {
        console.error('Network error - unable to reach server');
      }
    } else {
      // Error setting up request
      console.error('Request setup error:', error.message);
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
