# Integrations

Status: Accepted

The first real sources are manual entry, CSV import and Excel import. Open
Finance should initially be represented by a mock adapter shaped around the Open
Finance Chile specification.

All integrations produce source records and snapshots before any ledger
transaction is created or linked. Provider-specific payloads belong in snapshots
or adapter layers, not in the core domain model.

## Boundary rules

- Integrations must not directly mutate ledger transactions.
- External IDs are matching evidence, not permanent domain identity.
- Provider records may be missing IDs, reuse IDs or change IDs.
- Provider mutations must create traceable source history and review workflows.
- Discarded or hidden source records need tombstones so later syncs do not bring
  them back accidentally.

## Deferred adapters

- Real Open Finance Chile providers.
- Fintual or other public investment APIs.
- Splitwise.
- Aggregators or bank-specific adapters.

