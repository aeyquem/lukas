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
  group_id: string
  archived: boolean
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
  available_to_budget_breakdown: AvailableToBudgetBreakdown
  activity: BudgetValue
  uncategorized_activity: BudgetValue
  complete: boolean
  closed?: boolean
  categories: BudgetCategory[]
  archived_categories: BudgetCategory[]
  groups: BudgetGroup[]
}

export interface AvailableToBudgetBreakdown {
  prior_unassigned: BudgetValue
  income: BudgetValue
  openings: BudgetValue
  transfers: BudgetValue
  valuation: BudgetValue
  corrections: BudgetValue
  category_releases: BudgetValue
  category_rollover: BudgetValue
  assignments: BudgetValue
  funding_total: BudgetValue
}

export interface BudgetGroup {
  id: string
  name: string
  sort_order: number
  archived: boolean
  revision: number
}

export interface ExpenseCategory {
  id: string
  slug: string
  name: string
  sort_order: number
  archived: boolean
  group_id: string
  revision: number
  can_delete: boolean
}

export interface CategoryWrite {
  name: string
  sort_order: number
  group_id?: string
  archived?: boolean
  expected_revision?: number
}

export interface BudgetGroupWrite {
  name: string
  sort_order: number
  archived?: boolean
  expected_revision?: number
}

export interface CategoryDraft {
  name: string
  sort_order: string
  archived: boolean
  group_id?: string
  expected_revision?: number
}

export interface BudgetClosePreview extends BudgetSnapshot {
  close: {
    eligible: boolean
    reason: string | null
    expected_period_revision: string
    categories: Array<{ category_id: string; proposed_disposition: 'carry' | 'release' }>
  }
}

export interface BudgetMutationReceipt {
  resource_id: string
  revision: number
}
