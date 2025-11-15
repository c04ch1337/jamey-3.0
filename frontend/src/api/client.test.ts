import { describe, it, expect, vi, beforeEach } from 'vitest';
import { z } from 'zod';

const mockApiClient = {
  post: vi.fn(),
  get: vi.fn(),
  interceptors: {
    request: { use: vi.fn() },
    response: { use: vi.fn() },
  },
};

vi.doMock('axios', () => ({
  default: {
    create: () => mockApiClient,
  },
}));

describe('API Client', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // Dynamically import the client after mocks are set up
  const getClient = async () => (await import('./client'));

  describe('evaluateAction', () => {
    it('should return a valid response for a valid action', async () => {
      const { evaluateAction } = await getClient();
      const mockResponse = { data: { score: 0.9, action: 'test' } };
      mockApiClient.post.mockResolvedValue(mockResponse);

      const result = await evaluateAction('test');
      expect(result).toEqual(mockResponse.data);
      expect(mockApiClient.post).toHaveBeenCalledWith('/evaluate', { action: 'test' });
    });

    it('should throw an error for an empty action', async () => {
      const { evaluateAction } = await getClient();
      await expect(evaluateAction('')).rejects.toThrow('Action cannot be empty');
    });

    it('should throw for an action that is too long', async () => {
      const { evaluateAction } = await getClient();
      const longAction = 'a'.repeat(1001);
      await expect(evaluateAction(longAction)).rejects.toThrow('Action too long (max 1000 characters)');
    });

    it('should throw a Zod error for an invalid response', async () => {
      const { evaluateAction } = await getClient();
      const invalidResponse = { data: { score: 'not-a-number' } };
      mockApiClient.post.mockResolvedValue(invalidResponse);

      await expect(evaluateAction('test')).rejects.toThrow(z.ZodError);
    });
  });

  describe('getRules', () => {
    it('should return a list of moral rules', async () => {
      const { getRules } = await getClient();
      const mockRules = [{ name: 'rule1', description: 'desc1', weight: 50 }];
      const mockResponse = { data: mockRules };
      mockApiClient.get.mockResolvedValue(mockResponse);

      const result = await getRules();
      expect(result).toEqual(mockRules);
      expect(mockApiClient.get).toHaveBeenCalledWith('/rules');
    });

    it('should throw a Zod error for invalid rule data', async () => {
      const { getRules } = await getClient();
      const invalidRules = [{ name: 'rule1', description: 'desc1', weight: 'heavy' }];
      const mockResponse = { data: invalidRules };
      mockApiClient.get.mockResolvedValue(mockResponse);

      await expect(getRules()).rejects.toThrow(z.ZodError);
    });
  });

  describe('addRule', () => {
    it('should post the validated rule', async () => {
      const { addRule } = await getClient();
      const newRule = { name: '  rule2  ', description: '  <script>alert("xss")</script>desc2  ', weight: 75 };
      const sanitizedRule = { name: 'rule2', description: 'scriptalert("xss")/scriptdesc2', weight: 75 };
      
      mockApiClient.post.mockResolvedValue({ data: {} });

      await addRule(newRule);
      
      expect(mockApiClient.post).toHaveBeenCalledWith('/rules', sanitizedRule);
    });

    it('should throw a Zod error for an invalid rule object', async () => {
      const { addRule } = await getClient();
      const invalidRule = { name: '', description: 'desc', weight: 101 };
      await expect(addRule(invalidRule)).rejects.toThrow(z.ZodError);
    });
  });
});