# Spec Delta

## Purpose

Defines how the owner creates, views, and edits financial accounts and their dated currency balances while preserving the original denomination of each opening.

## ADDED Requirements

### Requirement: Create accounts with dated currency openings
The system SHALL create an account with a name, type, base currency, budget-participation choice, currency wallets, signed dated openings, and any required foreign-liability CLP basis atomically. Supported types SHALL include bank, cash, savings, card, and investment; investment accounts default to tracking-only. Each opening amount SHALL retain its currency and SHALL NOT be converted merely because it differs from the account's base currency.

#### Scenario: Account and opening are created together
- **WHEN** the owner creates an account with a valid type, one currency wallet, and a dated opening
- **THEN** the account and opening are persisted together with the original amount, currency, and date

#### Scenario: Investment account starts tracking-only
- **WHEN** the owner creates an investment account without explicitly enabling budget participation
- **THEN** the account is created as tracking-only and its balances do not fund the budget

#### Scenario: Account creation is rejected
- **WHEN** any account, wallet, opening, or required liability-basis value is invalid
- **THEN** no part of the account creation is persisted and field-level errors identify invalid input

### Requirement: Restrict wallet currencies by account type
Bank, savings, and investment accounts SHALL have exactly one currency wallet. Card and cash accounts MAY have multiple currency wallets. Cash held in different currencies MAY also be represented by separate cash accounts.

#### Scenario: Card account has multiple currency openings
- **WHEN** the owner creates a card account with more than one tracked currency balance
- **THEN** the system creates the account, wallets, and opening entries together and shows each original amount and date

#### Scenario: Cash account has multiple currency openings
- **WHEN** the owner creates a cash account with more than one tracked currency balance
- **THEN** the system creates the account, wallets, and opening entries together and shows each original amount and date

#### Scenario: Single-currency account types have one currency
- **WHEN** the owner creates a bank, savings, or investment account
- **THEN** the account has exactly one tracked currency wallet

#### Scenario: Single-currency account type requests multiple currencies
- **WHEN** the owner submits a bank, savings, or investment account with more than one currency wallet
- **THEN** the system rejects the request with a field-level account currency-count error and persists no part of the account

### Requirement: Read and edit accounts
The system SHALL show workspace accounts and their separately denominated balances. Editable descriptive values SHALL update under an expected revision; budget participation is fixed at creation in this slice. Opening entries count as activity for account type, base currency, and wallet currency changes, so those values SHALL be immutable after account creation.

#### Scenario: Account list is loaded
- **WHEN** the owner requests accounts
- **THEN** the response contains only workspace accounts, their tracked currencies, and authoritative balances

#### Scenario: Stale account edit is submitted
- **WHEN** an account edit uses an older revision
- **THEN** the system returns a stable stale-revision conflict without overwriting the account

### Requirement: Correct foreign opening liabilities with an explicit CLP basis
The system SHALL preserve a foreign opening liability's signed original-currency amount and separately supplied or confirmed signed CLP funding basis. A principal-changing correction SHALL update both values atomically and SHALL NOT derive the basis from daily FX.

#### Scenario: Foreign liability correction includes a confirmed basis
- **WHEN** the owner corrects foreign opening principal and supplies or confirms its signed CLP basis
- **THEN** the principal and basis are updated together with their provenance and revision

#### Scenario: Foreign liability correction omits its basis
- **WHEN** a correction changes foreign opening-liability principal without a paired signed CLP funding basis
- **THEN** the correction is rejected and the prior financial facts remain intact
