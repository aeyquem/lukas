# First-slice implementation readiness

Status: **ready for implementation**. The independent reviewer passed the
decision record on 2026-11-05. The owner asked orchestration to resolve missing
first-slice decisions without further questions and delegated technical choices
to a technical committee. See
`desc/13-first-slice-decisions.md` for the selected product rules and technical
baseline. This orchestration handoff does not itself start application code.

## Resolved first-slice decisions

- [x] Scope and interaction outcome: accounts, manual income/expense/transfer,
  current-month budget, and explicit month close per `desc/08-first-slice-plan.md`.
- [x] Product references reviewed: Actual Budget, Lunch Money, and YNAB inform
  dated openings, linked transfers, explicit rollover explanations, and
  correction visibility. Their differing automatic rollover and reconciliation
  behavior does not override Lukas's accepted behavior. See the research
  summary in `desc/13-first-slice-decisions.md` and its agent report.
- [x] Budget attribution: actual transaction date, `America/Santiago`, one
  budget period per transaction; no assigned-period override in this slice.
- [x] Funding derivation, transfers, cards, refunds, foreign assets, incomplete
  foreign expenses, opening liabilities, category remainders, and close
  checkpoints: `desc/13-first-slice-decisions.md`.
- [x] Month close: chronological first unclosed month only, after its calendar
  month ends and required values are complete; atomic snapshot and per-category
  signed remainder choice; unassigned ready-to-assign carries automatically.
- [x] Edit/correction scope: supported fields, immutable account metadata
  conditions, date corrections, opening treatments, closed-period correction
  routing, revisions, and no-value-history rule are specified in
  `desc/13-first-slice-decisions.md`.
- [x] Money/date wire profile, PostgreSQL constraints, API error contract,
  workspace bootstrap, idempotency, locking, retries, migration/runtime
  behavior, and API-by-use-case approach are specified in
  `desc/13-first-slice-decisions.md`.
- [x] Frontend interaction and first-slice manual accessibility review,
  pagination sizes, local Compose service map, same-machine exposure, route
  handling, runtime API configuration, and independent production images are
  specified in `desc/13-first-slice-decisions.md`.

## Audit gate

- [x] Independent reviewer audits `desc/13-first-slice-decisions.md` against
  the accepted product context, financial consistency, technical feasibility,
  and this tracker. Resolve findings with the writer and re-review until the
  reviewer passes the record.
- [x] After the audit passed, update this status to **ready for implementation**
  and hand the reviewed decision record to development. This orchestration
  request does not start application coding.

## Required implementation evidence (not planning blockers)

During implementation, validate the selected design with the real local
Compose path and PostgreSQL, not only mocks:

- Fresh and subsequent migrations, idempotent/concurrent workspace bootstrap,
  restart persistence, database outage/readiness, and bounded shutdown.
- Atomic account/opening creation, transfers, opening choices, assignments,
  edits, corrections, close, revisions, and idempotent retry behavior.
- Decimal/date parsing and round-trips, exact aggregates, range/overflow
  failures, incomplete foreign values, and stale-rate behavior.
- Browser → dev proxy → API → PostgreSQL read/write/read-back; routing, HMR,
  Host/Origin checks, paginated boundaries, keyboard flow, and NVDA with
  Firefox. Include Firefox, Chrome, and a Zen smoke check.
- Failure injection for partial writes and lost responses; demonstrate exact
  retry and retained draft behavior.

## Release gates and later slices

Before self-hosted release, define PostgreSQL provisioning/storage,
backup/restore and rehearse restoration; production proxy/TLS, secrets, logs,
diagnostics, image versioning, and schema rollout compatibility.

Before later features, separately define the CLI contract, imports and
recovery, reconciliation, advanced rollover, goals/pools/projections,
forecast matching, FX scenarios, multi-member access, hosted identity/SaaS,
and any SQLite mode. None blocks the first web slice. Do not turn unresolved
later product questions into first-slice requirements.
