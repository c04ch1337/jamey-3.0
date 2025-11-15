import axios from 'axios';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';
const API_KEY = import.meta.env.VITE_API_KEY;

export const apiClient = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
    // Include API key if available (supports both x-api-key and Authorization: Bearer formats)
    ...(API_KEY && { 'x-api-key': API_KEY }),
  },
});

// Add request interceptor for error handling
apiClient.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      console.error('Authentication failed. Please check your API key.');
    } else if (error.response?.status === 429) {
      console.error('Rate limit exceeded. Please slow down your requests.');
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

// API functions
export const evaluateAction = async (action: string): Promise<EvaluateResponse> => {
  const response = await apiClient.post<EvaluateResponse>('/evaluate', { action });
  return response.data;
};

export const getRules = async (): Promise<MoralRule[]> => {
  const response = await apiClient.get<MoralRule[]>('/rules');
  return response.data;
};

export const addRule = async (rule: AddRuleRequest): Promise<void> => {
  await apiClient.post('/rules', rule);
};

