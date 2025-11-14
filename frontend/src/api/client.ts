import axios from 'axios';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

export const apiClient = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

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

