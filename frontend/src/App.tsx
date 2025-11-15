import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { evaluateAction, getRules, addRule, type MoralRule, type AddRuleRequest, AddRuleRequestSchema } from './api/client'
import { z } from 'zod'
import './App.css'

// Input validation schema
const ActionSchema = z.string()
  .min(1, 'Action cannot be empty')
  .max(1000, 'Action too long (max 1000 characters)');

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
        <section className="evaluation-section">
          <h2>Evaluate Action</h2>
          <div className="input-group">
            <textarea
              value={action}
              onChange={(e) => setAction(e.target.value)}
              placeholder="Enter an action to evaluate..."
              rows={4}
            />
            <button onClick={handleEvaluate} disabled={!action.trim() || evaluateMutation.isPending}>
              {evaluateMutation.isPending ? 'Evaluating...' : 'Evaluate'}
            </button>
          </div>

          {evaluateMutation.isSuccess && (
            <div className="result">
              <h3>Evaluation Result</h3>
              <p><strong>Action:</strong> {evaluateMutation.data.action}</p>
              <p><strong>Moral Score:</strong> {evaluateMutation.data.score.toFixed(2)}</p>
              <div className="score-bar">
                <div
                  className="score-fill"
                  style={{ width: `${Math.min(100, (evaluateMutation.data.score / 10) * 100)}%` }}
                />
              </div>
            </div>
          )}

          {actionError && (
            <div className="error">
              {actionError}
            </div>
          )}
          {evaluateMutation.isError && (
            <div className="error">
              <strong>Error:</strong> {
                evaluateMutation.error instanceof Error 
                  ? evaluateMutation.error.message 
                  : 'Failed to evaluate action. Please try again.'
              }
              {evaluateMutation.error && 'response' in evaluateMutation.error && (
                <span> (Status: {(evaluateMutation.error as any).response?.status})</span>
              )}
            </div>
          )}
        </section>

        <section className="rules-section">
          <h2>Moral Rules</h2>
          {rulesLoading ? (
            <p>Loading rules...</p>
          ) : rulesError ? (
            <div className="error">
              <strong>Error loading rules:</strong> {
                rulesErrorDetails instanceof Error 
                  ? rulesErrorDetails.message 
                  : 'Failed to load rules. Please refresh the page.'
              }
            </div>
          ) : (
            <div className="rules-list">
              {rules.length === 0 ? (
                <p>No rules defined yet.</p>
              ) : (
                rules.map((rule) => (
                  <div key={rule.name} className="rule-card">
                    <h3>{rule.name}</h3>
                    <p>{rule.description}</p>
                    <span className="weight">Weight: {rule.weight}</span>
                  </div>
                ))
              )}
            </div>
          )}

          <div className="add-rule-form">
            <h3>Add New Rule</h3>
            <input
              type="text"
              placeholder="Rule name"
              value={newRule.name}
              onChange={(e) => setNewRule({ ...newRule, name: e.target.value })}
            />
            <input
              type="text"
              placeholder="Description"
              value={newRule.description}
              onChange={(e) => setNewRule({ ...newRule, description: e.target.value })}
            />
            <input
              type="number"
              placeholder="Weight"
              value={newRule.weight}
              onChange={(e) => setNewRule({ ...newRule, weight: parseFloat(e.target.value) || 0 })}
              step="0.1"
            />
            {ruleError && (
              <div className="error" style={{ marginTop: '0.5rem' }}>
                {ruleError}
              </div>
            )}
            {addRuleMutation.isError && (
              <div className="error" style={{ marginTop: '0.5rem' }}>
                <strong>Error:</strong> {
                  addRuleMutation.error instanceof Error 
                    ? addRuleMutation.error.message 
                    : 'Failed to add rule. Please try again.'
                }
              </div>
            )}
            <button onClick={handleAddRule} disabled={addRuleMutation.isPending}>
              {addRuleMutation.isPending ? 'Adding...' : 'Add Rule'}
            </button>
          </div>
        </section>
      </main>
    </div>
  )
}

export default App
