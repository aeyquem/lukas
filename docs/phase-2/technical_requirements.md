# Technical requirements to be discussed

These items are technical or architecture-facing and should be discussed in the
next phase before they are turned into implementation work. They should not be
mixed into the user-facing use-case spec unless they become visible product
behavior.

**1 Integration identity and history**

- [ ] S1.I1 Define stable internal IDs for accounts, transactions, sources,
  reports and external records.
  Why it matters: provider IDs may change or disappear. Stable internal IDs are
  required for deduplication, audit logs, API consumers and long-term history.
- [ ] S1.I2 Decide how to store external IDs when providers change them, omit
  them or reuse them.
  Why it matters: integrations cannot rely on a perfect external identifier. The
  storage model needs to support multiple IDs, missing IDs and ID changes.
- [ ] S1.I3 Define tombstones for deleted or hidden source transactions so syncs
  do not recreate dismissed records.
  Why it matters: without tombstones, a later sync may bring back transactions
  the user intentionally removed or resolved.
- [ ] S1.I4 Define how to preserve source payloads or source snapshots without
  coupling the domain model to a provider.
  Why it matters: raw data is useful for debugging and audits, but the core app
  should not depend on provider-specific JSON shapes.
- [ ] S1.I5 Define how to audit provider mutations of historical transactions.
  Why it matters: institutions can change pending or historical records. The app
  needs to track what changed, when it changed and what the user decided.

**2 Currency and conversion data**

- [ ] S2.I1 Model multiple conversion values per transaction: estimated,
  authorized, billed, paid, manual and reconciled.
  Why it matters: conversion values have different meanings and lifecycles. A
  single field would lose context and make reports harder to explain.
- [ ] S2.I2 Store conversion source and timestamp for each conversion value.
  Why it matters: the same rate from a bank, card network or estimate has
  different trust levels. Timestamps are needed for historic reporting.
- [ ] S2.I3 Decide how exchange-rate providers are selected when the institution
  does not provide the rate.
  Why it matters: missing rates need a fallback. The choice affects report
  accuracy, repeatability and user expectations.
- [ ] S2.I4 Decide how fees, spreads and international taxes are represented:
  separate transactions, conversion metadata or both.
  Why it matters: this changes reporting. Separate transactions make costs
  visible; metadata keeps the original operation compact.

**3 Period calculation**

- [ ] S3.I1 Define a period engine that supports fixed days, relative dates and
  manual overrides.
  Why it matters: budget periods are core to reports and projections. Hardcoding
  calendar months would block salary, card and custom reset rules.
- [ ] S3.I2 Decide how weekends and holidays affect period starts, payroll rules
  and recurring transactions.
  Why it matters: dates shift in real banking behavior. The app needs consistent
  rules so projected and actual periods can be compared fairly.
- [ ] S3.I3 Define period locking, reopening and post-close adjustments.
  Why it matters: reports need stable historical numbers, but late corrections
  happen. The technical model must support both.

**4 API / MCP**

- [ ] S4.I1 Define whether external access is a public API, MCP server or both.
  Why it matters: the interface choice affects authentication, tooling,
  deployment and how other agents or scripts will interact with the system.
- [ ] S4.I2 Define scoped permissions by workspace, account, report and action.
  Why it matters: financial data is sensitive. External access must be limited to
  the minimum data and actions each tool needs.
- [ ] S4.I3 Ensure writes from external tools are auditable and reversible.
  Why it matters: automation can make mistakes. Users need to know what changed
  and have a way to undo or correct it.
- [ ] S4.I4 Avoid exposing bank credentials, integration tokens or other secrets.
  Why it matters: external tools should never receive credentials that let them
  impersonate the user at a bank or provider.
- [ ] S4.I5 Define read-only access, rate limits and revocation from the start.
  Why it matters: these controls are harder to bolt on later and are important
  for privacy, safety and operational stability.

**5 Chile tax / annual summary addon**

- [ ] S5.I1 Decide whether the Chile annual summary belongs inside Lukas or
  should be a separate project that reads from it.
  Why it matters: tax reporting may have different scope, UX and risk than
  personal budgeting. The boundary affects architecture and maintenance.
- [ ] S5.I2 Model source-document traceability for tax-related data.
  Why it matters: users need to verify where each annual number came from. This
  is especially important when institutions provide their own certificates or
  reports.
- [ ] S5.I3 Decide how to detect duplicate tax/investment records across
  institutions.
  Why it matters: the same income or asset movement may appear in multiple
  documents. Double counting would make annual summaries unreliable.
- [ ] S5.I4 Support corrections after annual close without rewriting history.
  Why it matters: institutions may correct documents late. The system should
  preserve the original and the correction trail.
- [ ] S5.I5 Keep exports reviewable and avoid treating the output as official tax
  advice.
  Why it matters: this reduces product risk and keeps the feature positioned as
  preparation/support, not a replacement for user review or professional advice.
