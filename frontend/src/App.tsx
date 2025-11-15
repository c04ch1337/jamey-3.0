<<<<<<< HEAD
import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { evaluateAction, getRules, addRule, type MoralRule, type AddRuleRequest } from './api/client'
import './App.css'

function App() {
  const [action, setAction] = useState('')
=======
import { useState, useEffect } from 'react'
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

const RuleWeightSchema = z.number()
  .min(0, 'Weight must be >= 0')
  .max(100, 'Weight must be <= 100');

function App() {
  const [action, setAction] = useState('')
  const [actionError, setActionError] = useState<string | null>(null)
>>>>>>> origin/main
  const [newRule, setNewRule] = useState<AddRuleRequest>({
    name: '',
    description: '',
    weight: 8.0,
  })
<<<<<<< HEAD
  const queryClient = useQueryClient()

  // Fetch rules
  const { data: rules = [], isLoading: rulesLoading } = useQuery({
=======
  const [ruleError, setRuleError] = useState<string | null>(null)
  const [ruleSuccess, setRuleSuccess] = useState(false)
  const queryClient = useQueryClient()

  // Fetch rules
  const { 
    data: rules = [], 
    isLoading: rulesLoading, 
    isError: rulesError,
    error: rulesErrorDetails 
  } = useQuery({
>>>>>>> origin/main
    queryKey: ['rules'],
    queryFn: getRules,
  })

  // Evaluate action mutation
  const evaluateMutation = useMutation({
    mutationFn: evaluateAction,
    onSuccess: () => {
<<<<<<< HEAD
      // Optionally refetch rules or other data
=======
      // Clear any previous errors
      setActionError(null)
    },
    onError: (error) => {
      setActionError(
        error instanceof Error
          ? error.message
          : 'Failed to evaluate action. Please try again.'
      )
>>>>>>> origin/main
    },
  })

  // Add rule mutation
  const addRuleMutation = useMutation({
    mutationFn: addRule,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['rules'] })
      setNewRule({ name: '', description: '', weight: 8.0 })
<<<<<<< HEAD
    },
  })

  const handleEvaluate = () => {
    if (action.trim()) {
      evaluateMutation.mutate(action)
    }
  }

  const handleAddRule = () => {
    if (newRule.name && newRule.description) {
      addRuleMutation.mutate(newRule)
=======
      setRuleError(null)
      setRuleSuccess(true)
      // Clear success message after 3 seconds
      setTimeout(() => setRuleSuccess(false), 3000)
    },
    onError: (error) => {
      setRuleSuccess(false)
      setRuleError(
        error instanceof Error
          ? error.message
          : 'Failed to add rule. Please try again.'
      )
    },
  })

  // Handle form submission on Enter key
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
        const target = e.target as HTMLElement
        if (target.tagName === 'TEXTAREA' && target.id === 'action-input') {
          e.preventDefault()
          handleEvaluate()
        }
      }
    }
    document.addEventListener('keydown', handleKeyDown)
    return () => document.removeEventListener('keydown', handleKeyDown)
  }, [action])

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
    setRuleSuccess(false)
    
    // Validate all fields
    const nameValidation = RuleNameSchema.safeParse(newRule.name.trim())
    if (!nameValidation.success) {
      setRuleError(nameValidation.error.errors[0]?.message || 'Invalid rule name')
      return
    }

    const descValidation = RuleDescriptionSchema.safeParse(newRule.description.trim())
    if (!descValidation.success) {
      setRuleError(descValidation.error.errors[0]?.message || 'Invalid description')
      return
    }

    const weightValidation = RuleWeightSchema.safeParse(newRule.weight)
    if (!weightValidation.success) {
      setRuleError(weightValidation.error.errors[0]?.message || 'Invalid weight')
      return
    }

    // Validate complete rule object
    const validation = AddRuleRequestSchema.safeParse({
      name: nameValidation.data,
      description: descValidation.data,
      weight: weightValidation.data,
    })
    
    if (!validation.success) {
      setRuleError(validation.error.errors[0]?.message || 'Invalid input')
      return
    }

    addRuleMutation.mutate(validation.data)
  }

  const handleRuleFormKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault()
      handleAddRule()
>>>>>>> origin/main
    }
  }

  return (
    <div className="app">
      <header className="app-header">
        <h1>Jamey 3.0 - General & Guardian</h1>
        <p>Conscience Engine & Memory System</p>
      </header>

      <main className="app-main">
<<<<<<< HEAD
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
=======
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
              onKeyDown={(e) => {
                if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
                  e.preventDefault()
                  handleEvaluate()
                }
              }}
              placeholder="Enter an action to evaluate... (Ctrl+Enter to submit)"
              rows={4}
              aria-describedby="action-description"
              aria-invalid={!!actionError}
              aria-required="true"
              disabled={evaluateMutation.isPending}
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
>>>>>>> origin/main
                <div
                  className="score-fill"
                  style={{ width: `${Math.min(100, (evaluateMutation.data.score / 10) * 100)}%` }}
                />
              </div>
            </div>
          )}

<<<<<<< HEAD
          {evaluateMutation.isError && (
            <div className="error">
              Error evaluating action. Please try again.
=======
          {actionError && (
            <div className="error" role="alert" aria-live="assertive">
              {actionError}
            </div>
          )}
          {evaluateMutation.isError && !actionError && (
            <div className="error" role="alert" aria-live="assertive">
              <strong>Error:</strong> {
                evaluateMutation.error instanceof Error
                  ? evaluateMutation.error.message
                  : 'Failed to evaluate action. Please try again.'
              }
              {evaluateMutation.error && 'response' in evaluateMutation.error && (
                <span> (Status: {(evaluateMutation.error as AxiosError).response?.status})</span>
              )}
>>>>>>> origin/main
            </div>
          )}
        </section>

<<<<<<< HEAD
        <section className="rules-section">
          <h2>Moral Rules</h2>
          {rulesLoading ? (
            <p>Loading rules...</p>
          ) : (
            <div className="rules-list">
              {rules.map((rule) => (
                <div key={rule.name} className="rule-card">
                  <h3>{rule.name}</h3>
                  <p>{rule.description}</p>
                  <span className="weight">Weight: {rule.weight}</span>
                </div>
              ))}
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
            <button onClick={handleAddRule} disabled={addRuleMutation.isPending}>
              {addRuleMutation.isPending ? 'Adding...' : 'Add Rule'}
            </button>
=======
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

          <div className="add-rule-form" role="form" aria-labelledby="add-rule-heading" onKeyDown={handleRuleFormKeyDown}>
            <h3 id="add-rule-heading">Add New Rule</h3>
            <label htmlFor="rule-name-input">
              Rule name <span aria-label="required">*</span>
            </label>
            <input
              id="rule-name-input"
              type="text"
              placeholder="Rule name"
              value={newRule.name}
              onChange={(e) => {
                setNewRule({ ...newRule, name: e.target.value })
                setRuleError(null)
                setRuleSuccess(false)
              }}
              aria-required="true"
              aria-describedby="rule-name-description"
              disabled={addRuleMutation.isPending}
            />
            <div id="rule-name-description" className="visually-hidden">
              Enter a name for the moral rule
            </div>
            
            <label htmlFor="rule-description-input">
              Description <span aria-label="required">*</span>
            </label>
            <input
              id="rule-description-input"
              type="text"
              placeholder="Description"
              value={newRule.description}
              onChange={(e) => {
                setNewRule({ ...newRule, description: e.target.value })
                setRuleError(null)
                setRuleSuccess(false)
              }}
              aria-required="true"
              aria-describedby="rule-description-description"
              disabled={addRuleMutation.isPending}
            />
            <div id="rule-description-description" className="visually-hidden">
              Enter a description for the moral rule
            </div>
            
            <label htmlFor="rule-weight-input">
              Weight (0-100) <span aria-label="required">*</span>
            </label>
            <input
              id="rule-weight-input"
              type="number"
              placeholder="Weight"
              value={newRule.weight}
              onChange={(e) => {
                const value = parseFloat(e.target.value) || 0
                setNewRule({ ...newRule, weight: value })
                setRuleError(null)
                setRuleSuccess(false)
              }}
              onBlur={(e) => {
                // Clamp value to valid range on blur
                const value = parseFloat(e.target.value) || 0
                const clamped = Math.max(0, Math.min(100, value))
                if (clamped !== value) {
                  setNewRule({ ...newRule, weight: clamped })
                }
              }}
              step="0.1"
              min="0"
              max="100"
              aria-required="true"
              aria-describedby="rule-weight-description"
              disabled={addRuleMutation.isPending}
            />
            <div id="rule-weight-description" className="visually-hidden">
              Enter a weight between 0 and 100 for the moral rule
            </div>
            {ruleError && (
              <div className="error" role="alert" aria-live="assertive" style={{ marginTop: '0.5rem' }}>
                {ruleError}
              </div>
            )}
            {ruleSuccess && (
              <div className="success" role="status" aria-live="polite" style={{ marginTop: '0.5rem' }}>
                ✓ Rule added successfully!
              </div>
            )}
            {addRuleMutation.isError && !ruleError && (
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
              disabled={addRuleMutation.isPending || !newRule.name.trim() || !newRule.description.trim()}
              aria-describedby="add-rule-button-description"
            >
              {addRuleMutation.isPending ? 'Adding...' : 'Add Rule'}
            </button>
            <div id="add-rule-button-description" className="visually-hidden">
              Click to add a new moral rule to the system (or press Ctrl+Enter)
            </div>
>>>>>>> origin/main
          </div>
        </section>
      </main>
    </div>
  )
}

export default App
