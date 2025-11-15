import React from 'react';
import { render, screen } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { describe, it, expect } from 'vitest';
import App from './App';

const queryClient = new QueryClient();

describe('App', () => {
  it('renders headline', () => {
    render(
      <React.StrictMode>
        <QueryClientProvider client={queryClient}>
          <App />
        </QueryClientProvider>
      </React.StrictMode>
    );
    const headline = screen.getByText(/Jamey 3.0 - General & Guardian/i);
    expect(headline).toBeInTheDocument();
  });
});