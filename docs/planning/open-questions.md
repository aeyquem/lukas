# Open questions

Status: Open Question

This file is the only active home for unresolved planning prompts. When a
question is answered, move the accepted decision into the relevant
`architecture/` or `engineering/` file and remove it from here.

## Architecture and domain

- Define exact boundaries between domain, application services, storage,
  importers and host adapters.
- Decide whether a CLI helper is useful later for import, debug or admin tasks.
- Define the remaining initial schema details for source records, conversions,
  goals, reserves, periods, reports and review items.
- Decide whether source snapshots live in SQLite initially or local encrypted
  files.
- Define local profile and unlock behavior.
- Define local database encryption, future cloud-sync encryption and key
  recovery.

## Imports and integrations

- Define adapter interfaces for file, mock Open Finance and future providers.
- Define the import pipeline from source data through normalization,
  deduplication, review and final ledger state.
- Define tombstones and discard records.
- Define retry, backoff and error states for sync/import jobs.
- Decide how provider ID changes, missing IDs and reused IDs are represented.

## Money, dates and reports

- Choose currency library and ISO currency handling.
- Define timezone policy for institutions and source-specific reporting rules.
- Define period-rule schemas for fixed days, relative dates, source-specific
  rules and manual overrides.
- Define period locking, reopening and post-close adjustment behavior.
- Decide whether reports are calculated on demand, cached, materialized or a
  mix.
- Define report export formats and reproducibility rules.

## Scenarios and jobs

- Decide whether projections are derived views, persisted scenario entities or
  both.
- Define how scenario assumptions are stored and compared to the baseline.
- Choose background job strategy for sync, report precomputation, recurring
  estimates, period close, exchange-rate refresh and notifications.
- Define retry policy and dead-letter handling for background jobs.

## External access and modules

- Decide whether future external access uses API, MCP or both.
- Define authentication, scoped permissions, rate limits, revocation and audit
  rules for external tools.
- Define whether the Chile annual summary belongs inside Lukas or should be a
  separate module/project that reads from Lukas.
- Define plugin/module boundaries, permissions and packaging.

## Operations and future SaaS

- Choose future hosting, database, object storage and worker providers.
- Define environment strategy for local, staging and production.
- Define operational logging, metrics, tracing, error reporting and support
  tooling.
- Define SaaS auth, workspace membership, recovery and billing only when hosted
  deployment becomes real.
