# Monthly Budget Specification

## Purpose

Provides an explainable monthly plan that derives available money from dated financial events and preserves each explicit period close as a checkpoint.

## Requirements

### Requirement: Seed editable expense categories
The system SHALL seed Vivienda, Alimentación, Transporte, Servicios y comunicaciones, Salud, Educación, Hogar, Vestuario y cuidado personal, Restaurantes y ocio, Suscripciones, Seguros e impuestos, Mascotas, Regalos y donaciones, Ahorro general, and Otros for the owner workspace. The owner SHALL be able to add, rename, reorder, and archive an unused category. Uncategorized expenses SHALL remain visible.

#### Scenario: Workspace is initialized
- **WHEN** the owner opens the application with a bootstrapped workspace
- **THEN** the accepted starter expense categories are available exactly once

#### Scenario: Category is archived while in use
- **WHEN** the owner attempts to archive a category referenced by financial activity or an assignment
- **THEN** the system rejects the archive and preserves the category and related facts

### Requirement: Derive current-month funding from dated events
The system SHALL calculate budget funding from eligible signed openings, recognized income, and distinct funding effects, never from current account balances. Eligible foreign assets require an applicable daily rate; eligible foreign opening liabilities require their separately supplied signed CLP basis. If a required value is unavailable, dependent funding SHALL be incomplete rather than partial or zero.

#### Scenario: Initial signed balances determine funding
- **WHEN** the first budget is calculated from eligible signed CLP openings
- **THEN** positive and negative balances are included once, while credit and overdraft limits are excluded

#### Scenario: Eligible foreign asset has no stored rate
- **WHEN** no current or stored usable rate exists for an eligible foreign asset
- **THEN** the affected funding and ready-to-assign values are unavailable with a reason and no partial total

#### Scenario: Eligible foreign liability lacks its CLP basis
- **WHEN** an eligible foreign opening liability has no signed CLP funding basis
- **THEN** funding and ready-to-assign are unavailable and daily debt FX is not substituted

#### Scenario: Account or opening is added after tracking starts
- **WHEN** an eligible account or opening entry is added after budget tracking has begun
- **THEN** its signed CLP funding value is added once in its open-period date, or as a net linked correction in the current open correction period when its opening date is already closed

#### Scenario: Existing opening amount is corrected
- **WHEN** an opening amount is changed after its original funding event was applied
- **THEN** the budget applies only the net signed funding delta once and does not replay the full opening amount

### Requirement: Apply foreign-asset revaluation deltas once
The system SHALL automatically reflect eligible foreign-asset valuation changes as distinct funding deltas with their rate source and effective date. Revaluation SHALL NOT replay the original receipt or transfer funding effect.

#### Scenario: Eligible foreign asset rate changes
- **WHEN** an eligible foreign asset's CLP carrying valuation changes
- **THEN** only the signed difference from its prior applied valuation affects funding once and the old receipt or transfer is not counted again

#### Scenario: Revaluation is recalculated
- **WHEN** a previously applied valuation is recalculated or corrected
- **THEN** the linked budget effect is updated to the net delta rather than adding the full valuation again

### Requirement: Attribute activity and funding to actual periods
The system SHALL attribute each transaction to exactly one budget period using its actual date and the `America/Santiago` calendar. Expenses and refunds change category activity once; transfers are not income or expense and affect funding only according to account eligibility and accepted valuation rules.

#### Scenario: Expense is posted
- **WHEN** a categorized expense has a confirmed CLP value and an actual date in an open period
- **THEN** that period's category activity increases once and the wallet effect is not deducted again from ready-to-assign

#### Scenario: Transfer crosses budget eligibility
- **WHEN** a transfer has one eligible side and one tracking-only side
- **THEN** funding changes only by the eligible side's accepted carrying or received value and the transfer remains a transfer

#### Scenario: Unvalued foreign expense is present
- **WHEN** an open period contains an expense with no confirmed CLP statement value
- **THEN** affected category activity and remaining are incomplete, known CLP subtotals and unresolved original amounts are shown, assignments remain savable, and period close is blocked

### Requirement: Assign money and show signed category results
The system SHALL allow nonnegative assignments in an open period and show authoritative ready-to-assign, activity, and signed category remaining. Over-assignment and negative category values SHALL remain visible and SHALL NOT be clamped or silently rejected.

#### Scenario: Assignment is saved
- **WHEN** the owner assigns a nonnegative amount to a category in an open period
- **THEN** the assignment is persisted and authoritative budget values are returned or refetched

#### Scenario: Assignment exceeds available funding
- **WHEN** assignments exceed available funding
- **THEN** the system preserves the assignment intent and reports negative ready-to-assign or over-assignment without inventing coverage

#### Scenario: Assignment is stale
- **WHEN** the owner submits an assignment against an outdated period revision
- **THEN** the system returns a stale-revision conflict and the client retains the local draft for review

### Requirement: Carry open periods forward without reseeding
The system SHALL calculate an open period from the latest explicit close and intervening events. Until an earlier period closes, signed category remainders carry provisionally in-category and unassigned ready-to-assign carries forward. Recalculation after an earlier close SHALL replay later events once and preserve later saved assignments.

#### Scenario: New month opens while prior month remains open
- **WHEN** the owner views or assigns in a new calendar month before closing the prior month
- **THEN** the new month is calculated forward provisionally and does not reseed from account balances

#### Scenario: Earlier period closes after a later assignment
- **WHEN** an earlier open period is closed after assignments exist in a later period
- **THEN** later periods recalculate from the new checkpoint without duplicating activity or changing saved assignments

### Requirement: Close periods explicitly and chronologically
The system SHALL close only the earliest unclosed period after its calendar month ends and when required values are complete. Closing SHALL atomically preserve a snapshot, the per-category signed remainder disposition, and the resulting checkpoint; unassigned ready-to-assign carries automatically and account balances are not reseeded.

#### Scenario: Eligible period is closed
- **WHEN** the owner closes the earliest unclosed, ended, complete period with one carry-or-release choice per category
- **THEN** the system records one immutable snapshot and the resulting checkpoint atomically

#### Scenario: Period is current, out of order, or incomplete
- **WHEN** the owner tries to close a current period, a later period before an earlier one, or a period with incomplete required values
- **THEN** the system rejects the close with a stable reason and creates no snapshot

#### Scenario: Closed period receives a new close command
- **WHEN** a new idempotency key attempts to close an already-closed period
- **THEN** the system returns a stable conflict while an exact retry of the original command returns its saved result

### Requirement: Preserve closed snapshots during corrections
The system SHALL preserve closed period snapshots. A correction affecting a closed period SHALL post only its net linked budget effect to the current open correction period, or the next unclosed period when the current period is unavailable.

#### Scenario: Closed-period expense amount changes
- **WHEN** a closed-period expense is corrected to a different amount
- **THEN** the closed snapshot remains unchanged and only the net difference is reflected once in the correction period

#### Scenario: Assignment is corrected after close
- **WHEN** the owner corrects an assignment belonging to a closed period
- **THEN** the closed snapshot remains unchanged and the net linked assignment effect is posted to the open correction period

### Requirement: Preview and submit a close using a revision
The system SHALL expose a close preview with ready-to-assign, each signed category remainder, completeness, and proposed per-category disposition. A close command SHALL include the expected period/checkpoint revision and one disposition per category.

#### Scenario: Close preview is stale
- **WHEN** the period changes after the owner reviewed its close preview
- **THEN** the close command returns a stale-revision conflict and does not create a snapshot

#### Scenario: Close succeeds
- **WHEN** the owner confirms a complete current preview with the required dispositions and idempotency key
- **THEN** the client receives command receipt metadata and refetches the authoritative snapshot and next open budget

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
