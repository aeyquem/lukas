# Spec Delta

## ADDED Requirements

### Requirement: Review monthly budget summaries
The budget page SHALL show the selected period's available-to-budget, income, non-rolling negative category remainder, and uncategorized expense values when the API provides them, preserving incomplete states and currency context.

#### Scenario: Budget summary values are complete
- **WHEN** the selected budget period has complete summary data
- **THEN** the page shows each requested summary with its period and currency

#### Scenario: Budget summary value is incomplete or absent
- **WHEN** a requested summary is incomplete or unavailable in the API response
- **THEN** the page identifies the value as incomplete or unavailable rather than displaying zero

### Requirement: Review category assignment and activity
The budget page SHALL show each category's assigned, used, and available values with an accessible usage or overuse indicator and access to its transactions when the API supports that relationship.

#### Scenario: Category has budget activity
- **WHEN** the owner reviews a category with activity
- **THEN** the page shows exact assigned, used, and available values and an indicator derived from those values

#### Scenario: Category activity cannot be valued
- **WHEN** the category's dependent CLP activity is incomplete
- **THEN** the page preserves the incomplete state and does not imply a complete usage percentage

### Requirement: Manage expense categories from the budget
The budget page SHALL expose category creation and editing, and SHALL expose archive only under the existing category-in-use protection.

#### Scenario: Owner edits a category
- **WHEN** the owner changes an editable category field
- **THEN** the page saves through the category API and refreshes authoritative budget data

### Requirement: Preserve budget period in the URL
The budget page SHALL represent its selected month in navigable URL state independently of dashboard period context.

#### Scenario: Owner changes the budget month
- **WHEN** the owner selects a different budget month
- **THEN** the URL and loaded budget correspond to that month without changing the dashboard month
