# Use cases

Status: Accepted

This is the active product use-case index for the Lukas PoC.

## Product principles

- Chile first: CLP, local institutions, local formats and Chilean financial
  behavior are the first defaults.
- Multicurrency from the model: reports use a workspace base currency, but
  accounts and transactions keep their real currency.
- Manual before automatic: imported data must be reviewable and correctable.
- Integrations are adapters: the core app must not depend directly on one bank,
  fintech or provider shape.
- History is auditable: financial records keep source, change and reconciliation
  context.
- Projections are explainable: scenarios show which assumptions produced each
  result.

## Actors

- Primary user: person managing personal or household finances.
- Financial institution or provider: bank, card issuer, investment platform,
  Splitwise-like service or future Open Finance provider.
- External tool: future API/MCP client with scoped access.

## Use-case groups

- Transactions and reconciliation: `use-cases/transactions.md`
- Imports and integrations: `use-cases/imports-and-integrations.md`
- Budgeting, periods and currency: `use-cases/budgeting-and-periods.md`
- Goals and scenarios: `use-cases/goals-and-scenarios.md`
- Reports and external access: `use-cases/reports-and-external-access.md`
