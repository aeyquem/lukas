# Spec Delta

## ADDED Requirements

### Requirement: Summarize accounts by tracked currency
The accounts page SHALL identify each account with its name and type and show its authoritative net balance separately for every tracked currency.

#### Scenario: Account has one or more tracked wallets
- **WHEN** the owner views an account
- **THEN** its type and each wallet's exact currency balance are visible without cross-currency aggregation

### Requirement: Review transactions for a selected account
The accounts page SHALL provide a transaction list scoped to the selected account and preserve each transaction's actual date, currency, and type.

#### Scenario: Account is selected
- **WHEN** the owner selects an account with activity
- **THEN** the page displays that account's transactions and does not include activity from other accounts

#### Scenario: Selected account has no activity
- **WHEN** the owner selects an account without transactions
- **THEN** the page shows an account-specific empty state
