# Lukas implementation handoff

Status: **first-slice decisions reviewed and ready for development**.
The owner asked orchestration to resolve missing decisions without pausing for
questions and delegated technical decisions to a technical committee. This
planning task does not begin application implementation.

## Source of truth

Read `AGENTS.md`, then `desc/00-rewrite-context.md`. For the accepted first
web-slice scope, use `desc/05-mvp-acceptance.md` and
`desc/08-first-slice-plan.md`. The committee's concrete decisions and the
product-reference findings are consolidated in
`desc/13-first-slice-decisions.md`. `desc/12-pre-implementation-checklist.md`
tracks the audit gate, implementation evidence, release gates, and later
slices. `desc/09-technical-stack-proposal.md` remains broader context;
`desc/10-backend-architecture.md` and `desc/11-frontend-architecture.md`
retain implementation detail, with the decision record controlling any
resolved first-slice point that they previously marked open.

The old PoC documentation is historical and must not be used as current
implementation requirements. Preserve provenance, reviewable corrections,
decimal-safe money, actual dates, period context, and explainable projections.
Do not pull unresolved goals, pools, forecasts, imports, reconciliation, CLI,
or hosted identity behavior into the first slice.

## Reviewed first slice

The slice is accounts with dated signed opening balances, manual income,
expense and transfer activity, current-month budget assignments, and explicit
month close. It includes the accepted same-machine owner-only boundary,
PostgreSQL, a Rust backend, a Vue web client, independent backend/frontend
Dockerfiles, and local Docker Engine Compose. The owner may manage personal or
household finances; separate member identities are later scope. The CLI is a
planned interface, but its contract is required only before its later
implementation slice. MCP is deprecated.

The first slice preserves the accepted product details in
`desc/08-first-slice-plan.md`, including foreign opening liability funding
basis, incomplete foreign expense behavior, same-day/pre-opening choices,
provisional signed category carry, and no value history. The detailed period,
edit, API, money, runtime, frontend, and deployment contracts are in
`desc/13-first-slice-decisions.md`.

## Technical committee decision

Backend/persistence, finance-ledger, and frontend/deployment specialists
reviewed their areas. Their decisions are consolidated in
`desc/13-first-slice-decisions.md`. The chosen baseline is a readable one-crate
Rust modular monolith with Axum/Tokio, SQLx and PostgreSQL; Vue 3, strict
TypeScript and Vite; server-authoritative financial calculations; and the
accepted independent image and local Compose boundaries. Optional framework,
grid, chart, and UI-suite comparisons are not code gates.

## Writer/reviewer cycle and next action

The orchestration writer consolidated specialist proposals. The independent
reviewer passed the decision record after two corrections: a precise
once-only cross-currency transfer valuation rule and dated funding treatment
for account openings added after setup. Readiness is recorded in
`desc/12-pre-implementation-checklist.md`; runtime behavior remains to be
validated during development. This orchestration task did not start application
code. Self-hosted release operations and later feature contracts remain
tracked separately in the checklist and source documents.

## Rewrite base branch handoff

Status: **saving the rewrite baseline before implementation**. Preserve the
existing remote `main`; it contains a different history from the rewrite
working branch. Save the current clean-implementation rewrite state on a new
`rewrite-base` branch and use it as the parent of later feature branches.
Current changes include the intentional PoC removals, accepted rewrite context
and first-slice decisions, OpenSpec setup, and this handoff. The implementation
has not started.

The first implementation feature branch should use OpenSpec for the backend,
PostgreSQL, workspace bootstrap, health checks, local Compose path, and separate
backend/frontend Dockerfiles. Use a specialist writer and independent reviewer,
then have a tester validate the implementation and runtime evidence. Keep the
task status and next action current here so work can resume from the pushed base.

## Completed implementation: development foundation

Status: **implementation, integrated verification, independent reviewer/tester
audits, spec sync, and archive complete** (2026-11-05). The pushed base is
`origin/rewrite-base` at `b765807`. Work is isolated on `feature/dev-foundation`.

- OpenSpec change: archived at
  `openspec/changes/archive/2026-11-05-bootstrap-rust-postgres-dev-foundation/`
- Synced main specs: `openspec/specs/backend-runtime/spec.md` and
  `openspec/specs/container-deployment/spec.md`.
- Latest feature checkpoint: `feature/dev-foundation` is pushed to GitHub;
  implementation commit `4887fee` is followed by OpenSpec/handoff progress
  commits. The final integration test, spec sync, archive, and handoff edits are
  currently uncommitted.
- Planning artifacts: proposal, `backend-runtime` and `container-deployment`
  specs, design, and 11 completed implementation/evidence tasks are preserved
  in the archived change directory above.
- Plan review: independent backend/deployment reviewer passed; an additional
  task-level requirement for bounded startup/migration-lock, readiness,
  migrate-only, and shutdown-lifetime coverage was strictly validated and
  approved by the reviewer. The reviewer has now passed the integrated backend
  and frontend/container code review. The independent tester passed the latest
  offline and static checks.
- OpenSpec tracked progress: 11/11 tasks complete and archived. Backend, frontend, and Compose/proxy
  writers completed their assigned implementation. The reviewer passed backend
  and frontend/container implementation. The tester passed OpenSpec strict
  validation, frontend build/runtime-config checks, static Compose validation,
  Rust formatting/checking/test compilation, and all 13 library tests.
- Current source state: backend modules/migrations, the Vue/Vite shell and
  frontend image scripts, root Compose/proxy files, and setup documentation are
  present. Do not mark task checkboxes complete until their recorded checks
  pass.
- Local verification (2026-11-05): `cargo fmt --check`,
  `cargo check --all-targets --offline`,
  `cargo test --all-targets --no-run --offline`, and
  `cargo test --offline --lib -- --include-ignored` pass (13/13). `npm run
  build`, frontend runtime-config validation, and `openspec validate --all
  --strict` pass. PostgreSQL integration tests compile and run against live
  PostgreSQL below.
- Live runtime evidence (2026-11-05): the Compose stack reached healthy
  PostgreSQL/API status and exposed only `127.0.0.1:4173`. The PostgreSQL
  integration executable ran inside the API container: 7/7 passed, covering
  clean ordered migrations, repeat/concurrent workspace bootstrap, preservation
  of category edits, upgrade from the prior migration, held migration-lock
  deadline, readiness, and actual Origin middleware. The loader printed a
  non-fatal weak `GLIBC_2.39` version warning before the tests; the test process
  exited successfully. `--migrate-only` exited successfully while the normal
  API already occupied its configured port 8080, confirming migration-only did
  not bind a listener. Direct API POST checks returned 405 with canonical
  Origin and 403 with mismatched, missing, and forwarded-only Origin.

  Both production images built. The backend image was launched with `docker
  run` on a dedicated test network against a separate PostgreSQL container also
  launched with `docker run`; its `/health/ready` returned ready. The production
  frontend image was launched with `docker run` on the Compose private network;
  runtime config contained only `apiBaseUrl=/api`, proxied readiness returned
  ready, client routes returned 200 HTML, and `/api` plus unknown `/health`
  paths returned 404 rather than SPA content. Frontend runtime-config checks
  rejected injection-shaped values and confirmed secrets were not emitted.

  Through the development proxy, readiness returned ready; an invalid Host
  connection was closed; POST with canonical Origin reached the API (405),
  while mismatched and forwarded-only origins received 403; `/api` and
  `/health` remained outside SPA fallback; a client route returned 200 HTML;
  and HMR returned 101 Switching Protocols. PostgreSQL workspace/category
  counts stayed `1:15` across a container restart. Held-request server tests
  passed graceful drain, refusal of new requests, continued server lifetime
  beyond the shutdown timeout before signal, and forced exit at the deadline.
  While PostgreSQL was stopped,
  readiness returned HTTP 503 with `not_ready`; after restart it returned
  `ready`. `docker compose down` followed by `docker compose up --build -d`
  completed, the stack became healthy again, and persisted data remained. The
  only published port remained loopback `127.0.0.1:4173`. The feature stops at
  infrastructure; financial API/browser workflows remain later vertical-slice
  work. The independent reviewer and tester passed all 11/11 tasks and the
  verification record.
- Earlier environment note: Cargo, Node, npm, and Docker Compose are installed.
  The interactive sandbox lacked Docker Engine socket access, including its
  prior escalation attempt. The owner later explicitly authorized sudo Docker
  interaction and supplied live Compose output. A prolonged escalated Cargo
  dependency request was aborted by the user; do not retry network dependency
  resolution without a new request.
- Next: begin a separate vertical-slice change for financial API/browser
  workflows and keep the remaining first-slice checklist evidence in that scope.

Resume with `git switch feature/dev-foundation` and `openspec list --json` to
select or create the next implementation change.

## Completed implementation: first financial workflows

Status: **implementation, integrated verification, independent review, spec sync, and archive complete** (2026-11-06) on `feature/dev-foundation`. The OpenSpec change `first-financial-workflows` is archived at `openspec/changes/archive/2026-11-06-first-financial-workflows/`; all 20 implementation and evidence tasks are checked. Independent backend, frontend, and budget reviewers passed, and the recorded PostgreSQL/browser evidence is complete. `openspec list --json` reports no active changes. The completed implementation, synced specs, and handoff are present as uncommitted workspace changes; preserve them when preparing the next change.

 Account, transaction,
budget, idempotency-generation, and Vue workflow implementations are present.
The account edit decision is recorded in `desc/13-first-slice-decisions.md`:
account type, base currency, and wallet currencies become immutable as soon as
creation writes opening activity. `AGENTS.md` now directs privileged local
commands through the owner's `tmux attach -t Interactive` session and leaves
password entry to the owner.

The owner also selected the account currency rule: bank, savings, and
investment accounts have one currency wallet each; card and cash accounts may
have multiple currencies; cash may also be split across accounts. This is
recorded in the first-slice context, acceptance criteria, and active OpenSpec
change. The API rejects multi-wallet bank, savings, and investment creates.
The account form offers additional currencies for card and cash accounts and
preserves draft wallets if the account type changes.

Local validation passes: Rust formatting and test compilation, all 27 frontend
tests, the production frontend build, and strict validation of the active
OpenSpec change. Independent backend and frontend reviewers passed the latest
source/documentation snapshots, including the correction fixture updates. The
live PostgreSQL account API suite passed all 7 tests, including cash/card
multi-currency acceptance and single-currency restrictions for bank, savings,
and investment accounts. The transaction API suite passed all 9 tests after
the foreign-transfer fixture was given its required conversion provenance. The
budget API suite passes all 13 tests. A bug that omitted unresolved-only
categories from calculated category activity was fixed; its regression and the
complete budget suite now pass. Independent review of tasks 3.3–3.5 passed,
and category/budget API contracts are documented in `backend/API.md`.
The independent account reviewer passed this currency-rule implementation and
its updated requirements. OpenSpec task 2.5 is complete: the PostgreSQL account
API suite passed all 7 tests, and a Chromium 155.0.8059.84 run on 2026-11-05
created and read back `Browser check cash 20261105184653` (CLP 12500, USD 25.50)
and `Browser check card 20261105184653` (CLP 12500, EUR 25.50) through
`http://localhost:4173/accounts`. The same browser flow confirmed the form hides
the add-currency control for bank, savings, and investment, preserves a
two-wallet draft with an explanatory error when switched to bank, and permits
that draft when switched to cash. The two browser-check accounts remain in the
local development database.

Additional 2026-11-05 Chromium checks passed and have reproducible scripts in
`frontend/browser-checks/`:

- `accounts.mjs`: bank creation, required-name association, rename revision,
  opening correction, actual reload readback, and 390px viewport.
- `account-liability.mjs`: investment tracking-only default, created type
  immutability, and paired USD principal / signed CLP basis correction.
- `retry.mjs`: real committed write with deliberately discarded response,
  dismissal and reload persistence, exact retry and no duplicate account/opening.
- `persistence.mjs snapshot`, `sudo docker compose restart api`, then
  `persistence.mjs verify`: rejected invalid multiwallet bank with unchanged
  account list and exact readback of all nine accounts after API restart.
- `transactions.mjs`: income, expense, linked refund category inheritance,
  same-currency transfer, explicit same-day opening treatment and wallet delta.
- `runtime.mjs`: isolated database outage returned readiness503/liveness200;
  recovery returned200; invalid Host and missing/foreign mutation Origins failed.

Tasks4.2 and5.1 passed independent review and are checked. The browser helper
uses its own isolated Chromium profile/ephemeral debugging endpoint and verifies
the page URL before interacting. Clearly labeled browser records remain in the
local development database. Additional scenarios now use a separate
`lukas_browser_verify_20261105` database and `lukas-browser-api` container on
loopback4175 with a verification frontend proxy on loopback4174. Month closes
must run only against that isolated verification database.

Further isolated browser checks passed for categorized expenses, linked refund
inheritance, same- and cross-currency transfers with conversion provenance,
foreign expense confirmation and incomplete-value behavior, validation,
same-day opening treatment, correction revision/reason, and 259-record
deterministic pagination without gaps or duplicates. The isolated budget browser
flow passed category CRUD, incomplete foreign expense explanation, assignment
while incomplete, stale-draft retention, assignment and overassignment,
confirmed CLP valuation, signed carry and release, close preview and
confirmation, and closed-period assignment correction with next-period
recalculation. PostgreSQL read-back confirms the October close snapshot
(assignment 700, intent 750 after correction, activity 100), carry 200, and
November ready-to-assign 650. Its read-back script passes.

Firefox 158.0.1 and Zen 1.24b passed functional smoke checks on 2026-11-06
through the isolated proxy: account create/read-back, navigation and refresh
on the transaction, budget, and account routes, and a 390px viewport check.
Chromium 155.0.8059.84 passed the detailed account, transaction, and budget
workflows above. Google Chrome 157.0.8103.38 passed the same cross-browser smoke
on the development app, plus transaction entry/correction/pagination and budget
assignment/close-preview checks. Its smoke created account ID
`1f8681b3-b8ec-43aa-b31d-204f7b8baff2` and category ID
`946b5bc6-6a07-4080-9bd2-b4e31ca0772e`; the transaction run passed with 266
records rendered without pagination gaps. Formal accessibility review is
deferred until the final UI design and is not an MVP gate. The browser procedure
and recorded checks are in `frontend/BROWSER_TESTING.md`.

OpenSpec tasks 4.2–4.6 and 5.1–5.3 are checked. The independent budget reviewer
passed the integrated task 5.2 evidence.
Use only the isolated `lukas_browser_verify_20261105` database for further close
or FX experiments; do not repeat close mutations against the main development
DB.

The FX outage scenario seeded `Browser FX outage asset 2026-11` with an eligible
USD 100 opening. With Frankfurter reachable, the budget was complete with
funding CLP 96,757 and ready-to-assign CLP 97,457. The isolated API container's
`/etc/hosts` temporarily mapped `api.frankfurter.dev` to `127.0.0.1`; the budget
remained complete with identical values through the persisted stale-rate
fallback. PostgreSQL before/after read-back showed the same USD/CLP observation
(`db7e5217-587e-471a-a2f4-199c2a9baf37`, effective 2026-11-05, rate 968.07,
same observation timestamp) and one valuation effect of -2314 referencing that
observation. Restarting `lukas-browser-api` removed the temporary host override;
the recovered budget check passed with the same values. Reproducible checks are
`frontend/browser-checks/fx-outage-seed.mjs` and
`frontend/browser-checks/fx-outage-verify.mjs`. A subsequent PostgreSQL count
showed five USD/CLP observations and exactly one valuation effect for the test
wallet. The independent budget reviewer confirmed the combined browser and API
evidence satisfies task 5.2. Task 4.6 is complete with the Chrome smoke result
above.

Reproduction commands from the repository root (`sudo` commands run in
`tmux attach -t Interactive`):

```sh
LUKAS_BUDGET_CURRENT=2026-11 node frontend/browser-checks/fx-outage-seed.mjs
fx_wallet_id=$(node -p 'JSON.parse(require("node:fs").readFileSync("/tmp/lukas-fx-budget-baseline.json")).walletId')
inspect_fx() {
  sudo docker exec lukas-postgres-1 psql -U lukas -d lukas_browser_verify_20261105 -At -F '|' -c "SELECT 'usd_rate_count', count(*) FROM daily_fx_observations WHERE base_currency='USD' AND quote_currency='CLP'; SELECT 'latest_rate', id, effective_date, rate, observed_at FROM daily_fx_observations WHERE base_currency='USD' AND quote_currency='CLP' ORDER BY effective_date DESC, observed_at DESC LIMIT 1; SELECT 'valuation_effects', count(*), sum(amount), string_agg(rate_observation_id::text, ',') FROM budget_funding_effects WHERE wallet_id='$fx_wallet_id' AND period_start='2026-11-01' AND source_kind='valuation';"
}
inspect_fx # before outage
sudo docker exec --user 0 lukas-browser-api sh -c "printf '127.0.0.1 api.frankfurter.dev\\n' >> /etc/hosts"
LUKAS_BUDGET_API=http://localhost:4175 node frontend/browser-checks/fx-outage-verify.mjs outage
inspect_fx # after outage
sudo docker restart lukas-browser-api
LUKAS_BUDGET_API=http://localhost:4175 node frontend/browser-checks/fx-outage-verify.mjs recovered
```

## Ready to continue implementation

Next: select the next product slice before creating another OpenSpec change. The release checklist identifies self-hosted operations as remaining work: PostgreSQL provisioning and storage, backup/restore with a restoration rehearsal, production proxy/TLS, secrets, logs/diagnostics, image versioning, and schema rollout compatibility. These operational choices remain open; they are not yet a selected feature contract. The owner can either define that release-readiness slice next or choose a user-facing backlog item. Do not assume the broader goals, pools, reports, projections, imports, or CLI backlog is approved for the immediate next slice.
