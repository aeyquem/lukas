export interface BudgetValue {
  state: 'complete' | 'unavailable' | 'incomplete'
  amount?: string
  known_subtotal?: string
  reasons?: string[]
  unresolved?: Array<{ kind: string; category_id?: string | null; currency?: string | null; amount?: string | null }>
}

export interface BudgetCategory {
  id: string
  name: string
  sort_order: number
  assigned: string
  assignment_revision: number | null
  assignment_intent?: string | null
  activity: BudgetValue
  remaining: BudgetValue
}

export interface BudgetSnapshot {
  period: string
  revision: string
  funding: BudgetValue
  ready_to_assign: BudgetValue
  activity: BudgetValue
  uncategorized_activity: BudgetValue
  complete: boolean
  closed?: boolean
  categories: BudgetCategory[]
}

export interface ExpenseCategory {
  id: string
  slug: string
  name: string
  sort_order: number
  archived: boolean
}

export interface CategoryDraft {
  name: string
  sort_order: string
  archived: boolean
}

export interface BudgetClosePreview extends BudgetSnapshot {
  close: {
    eligible: boolean
    reason: string | null
    expected_period_revision: string
    categories: Array<{ category_id: string; proposed_disposition: 'carry' }>
  }
}

export interface BudgetMutationReceipt {
  resource_id: string
  revision: number
}
