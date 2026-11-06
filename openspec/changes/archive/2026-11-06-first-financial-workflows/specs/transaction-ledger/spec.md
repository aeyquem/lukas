# Spec Delta

## Purpose

Defines how the owner records, reviews, and corrects dated financial activity while preserving each original amount, currency, and relationship between linked entries.

## ADDED Requirements

### Requirement: Record manual income, expenses, and refunds
The system SHALL record income, expenses, and refunds as positive amounts in a tracked wallet currency and actual business date. Categories, payees, and notes are optional; refunds may link to a source expense.

#### Scenario: Refund is recorded
- **WHEN** the owner records a refund with its actual date and selected category
- **THEN** the wallet is credited and that period's category activity is reduced once without adding income or changing a closed snapshot

#### Scenario: Refund links to a source expense
- **WHEN** the owner links a refund to an existing expense
- **THEN** the refund retains that relationship and reduces the source expense category's activity in the refund's actual period once

### Requirement: Preserve foreign expenses with incomplete CLP values
The system SHALL retain an unvalued foreign expense in its original currency and mark dependent CLP budget values incomplete until a confirmed CLP statement value is supplied.

#### Scenario: Foreign expense lacks confirmed CLP value
- **WHEN** the owner records a foreign-currency expense without a confirmed CLP value
- **THEN** the original-currency wallet effect is recorded and affected CLP budget values show incompleteness and the unresolved original amount

#### Scenario: Confirmed CLP value completes an expense
- **WHEN** a confirmed CLP statement value is added to an existing unvalued foreign expense
- **THEN** the same expense becomes complete once without another wallet debit or expense

#### Scenario: Expense uses an untracked currency
- **WHEN** an expense is submitted in a currency not tracked by the selected account
- **THEN** the system rejects the request and does not create a transaction or wallet effect

### Requirement: Record transfers as one linked activity
The system SHALL record a transfer as one logical transaction with source and destination wallet effects, preserving both actual amounts, currencies, dates, and any supplied conversion evidence. A transfer SHALL NOT be reported as income or expense.

#### Scenario: Same-currency transfer is recorded
- **WHEN** the owner transfers a positive amount between tracked wallets of the same currency
- **THEN** both equal and opposite wallet effects are committed atomically under one logical transaction

#### Scenario: Cross-currency transfer is recorded
- **WHEN** the owner records a transfer with different source and received currencies or amounts
- **THEN** both actual amounts and their inspectable conversion relationship are retained and the transfer is not classified as income or expense

### Requirement: Resolve transactions relative to opening balances
The system SHALL require an explicit per-wallet choice to apply a transaction effect or offset the opening amount when the transaction date precedes the opening date or same-day inclusion is unknown. The system SHALL NOT infer ordering from identifiers or record order.

#### Scenario: Transaction precedes opening
- **WHEN** a transaction affects a wallet before its opening date
- **THEN** the system requests an apply-effect or preserve-current-balance choice before completing the write

#### Scenario: Transaction shares opening date with unknown inclusion
- **WHEN** a transaction and opening entry share a date and inclusion is unknown
- **THEN** the system requires the same explicit choice for each affected wallet

### Requirement: Edit and correct transactions safely
The system SHALL preserve actual transaction dates unless the owner explicitly corrects them, apply edits and affected wallet/budget effects atomically, and reject stale revisions. A correction SHALL request any newly unresolved opening treatment before it is applied.

#### Scenario: Transaction is corrected
- **WHEN** the owner explicitly corrects a transaction using its current revision
- **THEN** affected wallet and budget effects are replaced atomically and any newly unresolved opening treatment is requested

### Requirement: Financial mutations are idempotent
Each financial mutation SHALL require a client UUID idempotency key and an exact request. The system SHALL return the saved result for an exact retry and reject reuse of the key with different request content.

#### Scenario: Exact retry follows an unknown outcome
- **WHEN** the client retries the same method, route, key, and request after an unknown outcome
- **THEN** the system returns the original saved result without applying the mutation again

#### Scenario: Idempotency key is reused with changed content
- **WHEN** a key is presented with a different method, route, or request body
- **THEN** the system returns a stable conflict and applies no mutation

### Requirement: Preserve unknown writes for exact retry
Before sending a financial mutation, the browser SHALL persist its exact request bytes, method, route, idempotency key, and server-issued database-generation binding in origin storage. An unknown outcome SHALL remain available after alert dismissal and reload; Cancel SHALL NOT erase it, and Retry SHALL resend the exact command. If persistence fails, the browser SHALL NOT send the mutation.

#### Scenario: Pending command storage is unavailable
- **WHEN** the browser cannot persist the exact pending command before a write
- **THEN** it does not send the request and explains that the write was not attempted

#### Scenario: Write outcome remains unknown after reload
- **WHEN** a write times out and the owner dismisses the alert or reloads the page
- **THEN** the command remains available for exact retry with its original key and request

#### Scenario: Owner retries an unknown command
- **WHEN** the owner chooses Retry for a pending unknown command
- **THEN** the browser resends the saved method, route, body bytes, idempotency key, and generation binding without creating a new intent

### Requirement: Browse transactions with bounded pagination
The system SHALL return transactions in deterministic page order with a default page size of 50 and a maximum of 100, scoped to the owner workspace.

#### Scenario: Transaction page is requested
- **WHEN** the owner requests a transaction page without a page size
- **THEN** the system returns at most 50 workspace transactions and a continuation position when more records exist

#### Scenario: Maximum page size is exceeded
- **WHEN** a transaction page requests more than 100 records
- **THEN** the system rejects or bounds the request according to the documented API validation contract
