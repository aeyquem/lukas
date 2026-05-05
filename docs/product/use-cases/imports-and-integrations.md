# Imports and integrations

Status: Accepted

## Integration layer

Financial integrations use a common adapter boundary. Each source produces
source records and snapshots before ledger transactions are created or linked.

The product must support provider-specific fields without coupling the core
domain model to provider JSON, CSV or spreadsheet shapes.

## UC-003: Import transactions automatically

The user imports CSV or Excel files for one account at a time. The import flow
parses rows, stores raw and normalized snapshots, detects duplicates and sends
ambiguous records to review before final ledger changes.

Repeated or overlapping imports are expected and must be idempotent.

## UC-004: Connect with Open Finance Chile

The initial PoC uses a mock Open Finance adapter shaped around the Chilean Open
Finance direction. Real provider access is deferred until the app has stable
source-record, review and ledger behavior.

## UC-005: Connect with Fintual or public APIs

Future public financial APIs should feed the same adapter and source-record
pipeline as files and Open Finance providers.

Investment-specific fields belong in source snapshots or future feature modules
until the core model needs them for reports.

## UC-006: Connect with Splitwise

Future Splitwise-like integrations should import shared expenses as source
records and preserve enough context to explain reimbursements, participant
shares and settlements.

They must not bypass duplicate detection, review or ledger provenance.

