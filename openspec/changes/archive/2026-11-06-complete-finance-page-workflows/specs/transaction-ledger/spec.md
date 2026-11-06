# Spec Delta

## ADDED Requirements

### Requirement: Review transactions in a searchable table
The transactions page SHALL show loaded transactions in a semantic table with date, payee, category, tags, notes, incoming and outgoing amounts, reconciliation status, and an edit action; fields not yet tracked by the API SHALL be clearly identified as unavailable.

#### Scenario: Loaded transactions are displayed
- **WHEN** the owner opens the transactions page
- **THEN** each loaded row presents its actual date, payee, category, notes, currency-specific in/out amount, and available review fields

#### Scenario: Tag or reconciliation data is not available
- **WHEN** the API does not return tag or reconciliation state
- **THEN** the table identifies those fields as not tracked and does not imply that a transaction has no tags or is reconciled

### Requirement: Search and sort loaded transactions
The transactions page SHALL let the owner search and sort the currently loaded rows while making the current-page scope clear.

#### Scenario: Owner searches loaded rows
- **WHEN** the owner enters a search term
- **THEN** only loaded rows matching transaction text are shown and pagination remains available

#### Scenario: Owner sorts by a column
- **WHEN** the owner selects a sortable column heading
- **THEN** the currently loaded rows are sorted by that field and the active direction is exposed accessibly

### Requirement: Start transaction entry from the list
The transactions page SHALL provide a visible add-transaction action and an edit action for each transaction, using the existing transaction write workflow.

#### Scenario: Owner starts a transaction
- **WHEN** the owner activates the add or edit action
- **THEN** the existing transaction form opens with the appropriate create or edit state
