# Spec Delta

## MODIFIED Requirements

### Requirement: Seed editable expense categories
The system SHALL seed the accepted starter expense categories for the owner workspace. The owner SHALL be able to add, rename, reorder, and archive categories, and manage category groups. Archived categories SHALL remain available to historical records. The system SHALL permit permanent category deletion only when the category has no transactions or retained budget history.

#### Scenario: Workspace is initialized
- **WHEN** the owner opens the application with a bootstrapped workspace
- **THEN** the accepted starter expense categories and predefined budget groups are available exactly once

#### Scenario: Category is archived while in use
- **WHEN** the owner archives a category
- **THEN** it is removed from active budget planning while existing transactions, assignments, and closed snapshots retain their category reference and historical label

#### Scenario: Category with transactions is permanently deleted
- **WHEN** the owner attempts to permanently delete a category that has one or more transactions
- **THEN** the system rejects the deletion and preserves the category and related facts

#### Scenario: Unused category is permanently deleted
- **WHEN** the owner permanently deletes a category with no transactions or retained budget history
- **THEN** the system removes it from active category management

#### Scenario: Category with retained budget history is permanently deleted
- **WHEN** the owner attempts to permanently delete a category referenced by assignments, corrections, or closed snapshots
- **THEN** the system rejects deletion and preserves the category and retained references; archival remains available

### Requirement: Review monthly budget summaries
The budget page SHALL show the selected period's available-to-budget amount as its primary summary, with an explanation of the authoritative calculation and period/currency context. When category balances roll forward within their categories, the available-to-budget breakdown SHALL show zero from category rollover; those balances SHALL NOT be counted as available-to-budget. The page SHALL preserve incomplete or unavailable states.

#### Scenario: Budget summary values are complete
- **WHEN** the selected budget period has complete summary data
- **THEN** the page shows the available-to-budget amount and its available calculation details, including income and carried unassigned funds when supplied by the API

#### Scenario: Category balances roll forward within categories
- **WHEN** category remainders carry into the selected period within their respective categories
- **THEN** the available-to-budget breakdown shows zero from category rollover and does not add those balances to the available amount

#### Scenario: Explicit close releases category balances
- **WHEN** the existing period-close choice releases signed category balances to available-to-budget
- **THEN** the available-to-budget breakdown identifies the actual signed released amount separately from income and carried unassigned funds

#### Scenario: Budget summary value is incomplete or absent
- **WHEN** a requested summary value is incomplete or unavailable in the API response
- **THEN** the page identifies the value as incomplete or unavailable rather than displaying zero

### Requirement: Review category assignment and activity
The budget page SHALL show each active category's assigned, used, and available values with an accessible usage or overuse indicator and access to its transactions when the API supports that relationship. The assignment amount SHALL be editable directly in its table cell.

#### Scenario: Category has budget activity
- **WHEN** the owner reviews a category with activity
- **THEN** the page shows exact assigned, used, and available values and an indicator derived from those values

#### Scenario: Category activity cannot be valued
- **WHEN** the category's dependent CLP activity is incomplete
- **THEN** the page preserves the incomplete state and does not imply a complete usage percentage

#### Scenario: Owner edits an assignment inline
- **WHEN** the owner changes and saves a category's assignment amount in the table
- **THEN** the assignment is saved through the authoritative budget API and affected budget values are refreshed

### Requirement: Manage expense categories from the budget
The budget page SHALL expose category creation and editing through a category detail drawer, and SHALL expose archive and permanent deletion only under their respective history-protection rules.

#### Scenario: Owner edits a category
- **WHEN** the owner opens a category's details and changes an editable field
- **THEN** the page saves through the category API and refreshes authoritative budget data

#### Scenario: Owner archives a category from the drawer
- **WHEN** the owner archives a category in its detail drawer
- **THEN** the category is hidden from active budgeting and its historical references remain intact

## ADDED Requirements

### Requirement: Organize budget categories into groups
The budget page SHALL display active categories in named groups, including predefined groups at workspace initialization. The owner SHALL be able to add, rename, reorder, and archive groups.

#### Scenario: Owner reviews grouped categories
- **WHEN** the owner opens the selected budget period
- **THEN** active categories appear under their assigned groups in the saved order

#### Scenario: Owner manages groups
- **WHEN** the owner adds, renames, reorders, or archives a group
- **THEN** the updated group arrangement is saved and reflected in the budget

### Requirement: Lock category rearrangement outside edit mode
The budget page SHALL keep group and category rearrangement controls locked until the owner enters an explicit edit mode.

#### Scenario: Arrangement controls are locked
- **WHEN** the owner views the budget outside edit mode
- **THEN** groups and categories cannot be dragged or reordered

#### Scenario: Owner rearranges budget organization
- **WHEN** the owner enters edit mode and drags a group or category to a new position
- **THEN** the new order is saved and shown in the budget

### Requirement: Separate budget planning and category administration
The Presupuesto tab SHALL contain exactly one table titled “Presupuesto”, replacing the previous “Categorías” title. “Administrar categorías” SHALL be available in its own separate tab, preserving the category and group capabilities required by this change. Further refinement of the administration tab is deferred.

#### Scenario: Owner views budget planning
- **WHEN** the owner selects the Presupuesto tab
- **THEN** exactly one table is shown, titled “Presupuesto”, and category administration is available in a separate tab

#### Scenario: Owner opens category administration
- **WHEN** the owner selects the category administration tab
- **THEN** category administration is available without adding another table to the Presupuesto tab
