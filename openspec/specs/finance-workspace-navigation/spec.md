# Finance Workspace Navigation Specification

## Purpose

Provides clear navigation among finance workflows and a dashboard that summarizes authoritative workspace data without inventing totals or mixing report periods.

## Requirements

### Requirement: Navigate to finance areas
The application SHALL provide named destinations for the dashboard, budget, accounts, transactions, payees, projections, and objectives, with the active destination indicated to the user.

#### Scenario: Owner selects a supported destination
- **WHEN** the owner selects a finance area from primary navigation
- **THEN** the application navigates to that area's page and identifies the active destination

#### Scenario: Destination lacks an implemented data contract
- **WHEN** the owner opens payees, projections, or objectives before its data contract is available
- **THEN** the page explains that the feature is not yet available and does not present fabricated financial data

### Requirement: Choose dashboard period context
The dashboard SHALL let the owner choose a calendar month in the workspace timezone and SHALL keep that context distinct from other pages' period state.

#### Scenario: Dashboard period changes
- **WHEN** the owner chooses a different dashboard month
- **THEN** dashboard summaries use that month without changing the independently selected budget period

### Requirement: Summarize authoritative workspace data
The dashboard SHALL present account balances, selected-period budget totals, and recent transactions only from authoritative API responses, labeling each value with its currency and period where applicable.

#### Scenario: Dashboard data is available
- **WHEN** the dashboard successfully loads its source data
- **THEN** it displays the selected-period summary and recent activity with exact formatted values and links to their source pages

#### Scenario: Dashboard source data is unavailable
- **WHEN** one or more dashboard requests fail
- **THEN** the page identifies the unavailable section and does not substitute zero for missing data
