import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { evaluateAction, getRules, addRule, type AddRuleRequest, AddRuleRequestSchema } from './api/client'
import { z } from 'zod'
import type { AxiosError } from 'axios'
import './App.css'

// Input sanitization function
const sanitizeInput = (input: string): string => {
  return input
    .trim()
    .replace(/[<>]/g, '') // Remove potential HTML tags
    .replace(/javascript:/gi, '') // Remove javascript: protocol
    .replace(/on\w+=/gi, '') // Remove event handlers
    .slice(0, 1000); // Enforce max length
};

// Enhanced input validation schemas with sanitization
const ActionSchema = z.string()
  .min(1, 'Action cannot be empty')
  .max(1000, 'Action too long (max 1000 characters)')
  .transform(sanitizeInput);

const RuleNameSchema = z.string()
  .min(1, 'Rule name is required')
  .max(100, 'Rule name too long (max 100 characters)')
  .transform(sanitizeInput);

const RuleDescriptionSchema = z.string()
  .min(1, 'Description is required')
  .max(500, 'Description too long (max 500 characters)')
  .transform(sanitizeInput);

function App() {
  const [action, setAction] = useState('')
  const [actionError, setActionError] = useState<string | null>(null)
  const [newRule, setNewRule] = useState<AddRuleRequest>({
    name: '',
    description: '',
    weight: 8.0,
  })
  const [ruleError, setRuleError] = useState<string | null>(null)
  const queryClient = useQueryClient()

  // Fetch rules
  const { 
    data: rules = [], 
    isLoading: rulesLoading, 
    isError: rulesError,
    error: rulesErrorDetails 
  } = useQuery({
    queryKey: ['rules'],
    queryFn: getRules,
  })

  // Evaluate action mutation
  const evaluateMutation = useMutation({
    mutationFn: evaluateAction,
    onSuccess: () => {
      // Optionally refetch rules or other data
    },
  })

  // Add rule mutation
  const addRuleMutation = useMutation({
    mutationFn: addRule,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['rules'] })
      setNewRule({ name: '', description: '', weight: 8.0 })
    },
  })

  const handleEvaluate = () => {
    setActionError(null)
    
    // Validate input
    const validation = ActionSchema.safeParse(action.trim())
    if (!validation.success) {
      setActionError(validation.error.errors[0]?.message || 'Invalid input')
      return
    }

    evaluateMutation.mutate(validation.data)
  }

  const handleAddRule = () => {
    setRuleError(null)
    
    // Validate input
    const validation = AddRuleRequestSchema.safeParse(newRule)
    if (!validation.success) {
      setRuleError(validation.error.errors[0]?.message || 'Invalid input')
      return
    }

    addRuleMutation.mutate(validation.data)
  }

  return (
    <div className="app">
      <header className="app-header">
        <h1>Jamey 3.0 - General & Guardian</h1>
        <p>Conscience Engine & Memory System</p>
      </header>

      <main className="app-main">
        <section className="evaluation-section" aria-labelledby="evaluation-heading">
          <h2 id="evaluation-heading">Evaluate Action</h2>
          <div className="input-group">
            <label htmlFor="action-input" className="visually-hidden">
              Enter an action to evaluate
            </label>
            <textarea
              id="action-input"
              value={action}
              onChange={(e) => setAction(e.target.value)}
              placeholder="Enter an action to evaluate..."
              rows={4}
              aria-describedby="action-description"
              aria-invalid={!!actionError}
              aria-required="true"
            />
            <div id="action-description" className="visually-hidden">
              Enter the action you want to evaluate for moral implications
            </div>
            <button
              onClick={handleEvaluate}
              disabled={!action.trim() || evaluateMutation.isPending}
              aria-describedby="evaluate-button-description"
            >
              {evaluateMutation.isPending ? 'Evaluating...' : 'Evaluate'}
            </button>
            <div id="evaluate-button-description" className="visually-hidden">
              Click to evaluate the moral score of the entered action
            </div>
          </div>

          {evaluateMutation.isSuccess && (
            <div className="result" role="status" aria-live="polite">
              <h3>Evaluation Result</h3>
              <p><strong>Action:</strong> {evaluateMutation.data.action}</p>
              <p><strong>Moral Score:</strong> {evaluateMutation.data.score.toFixed(2)}</p>
              <div className="score-bar" role="progressbar" aria-valuenow={evaluateMutation.data.score} aria-valuemin={0} aria-valuemax={10} aria-label={`Moral score: ${evaluateMutation.data.score.toFixed(2)} out of 10`}>
                <div
                  className="score-fill"
                  style={{ width: `${Math.min(100, (evaluateMutation.data.score / 10) * 100)}%` }}
                />
              </div>
            </div>
          )}

          {actionError && (
            <div className="error" role="alert" aria-live="assertive">
              {actionError}
            </div>
          )}
          {evaluateMutation.isError && (
            <div className="error" role="alert" aria-live="assertive">
              <strong>Error:</strong> {
                evaluateMutation.error instanceof Error
                  ? evaluateMutation.error.message
                  : 'Failed to evaluate action. Please try again.'
              }
              {evaluateMutation.error && 'response' in evaluateMutation.error && (
                <span> (Status: {(evaluateMutation.error as AxiosError).response?.status})</span>
              )}
            </div>
          )}
        </section>

        <section className="rules-section" aria-labelledby="rules-heading">
          <h2 id="rules-heading">Moral Rules</h2>
          {rulesLoading ? (
            <p aria-live="polite">Loading rules...</p>
          ) : rulesError ? (
            <div className="error" role="alert" aria-live="assertive">
              <strong>Error loading rules:</strong> {
                rulesErrorDetails instanceof Error
                  ? rulesErrorDetails.message
                  : 'Failed to load rules. Please refresh the page.'
              }
            </div>
          ) : (
            <div className="rules-list" role="list" aria-label="Moral rules list">
              {rules.length === 0 ? (
                <p>No rules defined yet.</p>
              ) : (
                rules.map((rule) => (
                  <div key={rule.name} className="rule-card" role="listitem">
                    <h3>{rule.name}</h3>
                    <p>{rule.description}</p>
                    <span className="weight">Weight: {rule.weight}</span>
                  </div>
                ))
              )}
            </div>
          )}

          <div className="add-rule-form" role="form" aria-labelledby="add-rule-heading">
            <h3 id="add-rule-heading">Add New Rule</h3>
            <label htmlFor="rule-name-input" className="visually-hidden">
              Rule name
            </label>
            <input
              id="rule-name-input"
              type="text"
              placeholder="Rule name"
              value={newRule.name}
              onChange={(e) => setNewRule({ ...newRule, name: e.target.value })}
              aria-required="true"
              aria-describedby="rule-name-description"
            />
            <div id="rule-name-description" className="visually-hidden">
              Enter a name for the moral rule
            </div>
            
            <label htmlFor="rule-description-input" className="visually-hidden">
              Description
            </label>
            <input
              id="rule-description-input"
              type="text"
              placeholder="Description"
              value={newRule.description}
              onChange={(e) => setNewRule({ ...newRule, description: e.target.value })}
              aria-required="true"
              aria-describedby="rule-description-description"
            />
            <div id="rule-description-description" className="visually-hidden">
              Enter a description for the moral rule
            </div>
            
            <label htmlFor="rule-weight-input" className="visually-hidden">
              Weight
            </label>
            <input
              id="rule-weight-input"
              type="number"
              placeholder="Weight"
              value={newRule.weight}
              onChange={(e) => setNewRule({ ...newRule, weight: parseFloat(e.target.value) || 0 })}
              step="0.1"
              min="0"
              max="100"
              aria-required="true"
              aria-describedby="rule-weight-description"
            />
            <div id="rule-weight-description" className="visually-hidden">
              Enter a weight between 0 and 100 for the moral rule
            </div>
            {ruleError && (
              <div className="error" role="alert" aria-live="assertive" style={{ marginTop: '0.5rem' }}>
                {ruleError}
              </div>
            )}
            {addRuleMutation.isError && (
              <div className="error" role="alert" aria-live="assertive" style={{ marginTop: '0.5rem' }}>
                <strong>Error:</strong> {
                  addRuleMutation.error instanceof Error
                    ? addRuleMutation.error.message
                    : 'Failed to add rule. Please try again.'
                }
              </div>
            )}
            <button
              onClick={handleAddRule}
              disabled={addRuleMutation.isPending}
              aria-describedby="add-rule-button-description"
            >
              {addRuleMutation.isPending ? 'Adding...' : 'Add Rule'}
            </button>
            <div id="add-rule-button-description" className="visually-hidden">
              Click to add a new moral rule to the system
            </div>
          </div>
        </section>
      </main>
    </div>
  )
}

export default App
