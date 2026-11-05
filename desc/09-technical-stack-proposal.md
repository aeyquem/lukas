# Technical stack and architecture proposal

Status: **Current technical planning document.** It consolidates the parallel
technical proposal with the accepted rewrite direction and first-slice
decisions. It is authoritative for technical recommendations that have not
otherwise been settled; it is not blanket approval of every recommendation.
Precedence: direct owner decisions and accepted product behavior in
`desc/00-rewrite-context.md`, `desc/05-mvp-acceptance.md`,
`desc/08-first-slice-plan.md`, and `orchestrator.md` override conflicting
proposal text here. Sections marked “accepted direction” record a settled
direction; “recommendation”, “candidate”, and “open” remain subject to review.

This proposal was prepared from `desc/00-rewrite-context.md`,
`desc/01-idea.md`–`desc/08-first-slice-plan.md`, and the visual exploration in
`explore/`. The frontend and backend leads each received an independent
counter-review.

## Accepted rewrite constraints

- Build a Rust backend and web application. The first deployment model is
  self-hosted, with a later SaaS path kept possible.
- The backend and frontend have separate Dockerfiles. Docker Compose with the
  local Docker Engine is for development; production must not require Compose.
  The accepted self-hosted reference runs the frontend, backend and PostgreSQL
  on one machine. Operator-specific topology and database provisioning remain
  open.
- The first slice and initial self-hosted reference use PostgreSQL. SQLite is
  deferred as a possible later local desktop option.
- The initial instance is owner-only; the owner may manage personal or
  household finances. Separate member identities are later scope.
- A CLI for people and agents is planned for a later slice. MCP is deprecated
  for now. The CLI contract remains open and does not gate the first web slice.
- The first slice follows `desc/08-first-slice-plan.md`; broader product
  capabilities in this proposal do not silently expand its scope.

## Executive summary

| Area | Current proposal | Review status |
| --- | --- | --- |
| Frontend | **Vue 3 + strict TypeScript + Vite**, SFCs and scoped CSS; semantic HTML, basic CSS and small app-owned components | Owner-approved starting direction; framework comparison is optional and not a code gate |
| Backend | **Rust modular monolith with Axum/Tokio, SQLx, Serde, rust_decimal, chrono, tracing and thiserror** | Selected under the owner's delegation; see [backend architecture](10-backend-architecture.md) |
| Database | **PostgreSQL for the first slice; SQLx with explicit SQL and versioned migrations** | Accepted; migration, numeric and integration behavior need validation. SQLite is deferred |
| API | HTTP/JSON; explicit business-date, known transaction-time and decimal-string values; OpenAPI/client generation candidate | Prove one source of truth and generated-client quality before adopting the pipeline |
| Numeric representation | **`rust_decimal::Decimal` calculation envelope; PostgreSQL `NUMERIC` persistence** | Accepted for the first slice. Division and rounding policy still need definition |
| Jobs | No general job system initially; add resumable import work only if needed | Defer generalized queue/worker infrastructure |
| Deployment | Separate backend and frontend Dockerfiles; Compose for local development; self-hosted PostgreSQL reference and future hosted PostgreSQL reference | Accepted for now. Operator-specific proxy/TLS, storage provisioning, upgrades/backups, and SaaS boundary remain open |

The first implementation slice is the accounts, manual transactions, and
current-month budget scope in `desc/08-first-slice-plan.md`. Reports,
reconciliation, imports, rollover, goals and projections are later scope. The
selected backend architecture, code organization and readability conventions
are detailed in `desc/10-backend-architecture.md`.

The major cost driver is not framework choice by itself. It is the need to
preserve financial meaning across actual activity, budget intent, reporting,
source evidence and forecasts (`desc/07-domain-model.md` §1). Favor a small,
conventional stack with explicit invariants and worked-case tests over a large
framework or a generic abstraction layer.

## Frontend proposal

### Suggested starting stack

- **React + strict TypeScript**, built with **Vite**. Use conventional functional
  components and avoid an internal framework. A client-rendered static build
  fits a self-hosted web client; SSR is not justified by the current product
  docs.
- **React Router** for navigable views and shareable filters/period state.
- **CSS custom properties and layers**, with component-scoped CSS where useful.
  Treat `explore/base.css` as a token/style seed, not a completed design system.
  Consolidate it with the separate vocabulary in `explore/design-system.html`.
- **Native semantic HTML first.** Use a small set of **Radix UI** primitives
  when a complex dialog, menu or popover benefits from managed keyboard/focus
  behavior. Keep thin app-owned components and avoid a copied component suite.
- **React Hook Form + Zod** for complex form state and input feedback. These
  validate the interface, not financial rules; Rust/backend remains authoritative.
- **TanStack Query** for server state. Keep edits/drafts separate from cached
  server results, avoid optimistic changes to authoritative financial totals,
  and refresh affected data after successful writes. No global client store
  initially.
- Begin with **semantic tables and row/detail editors**. Add **TanStack Table**
  if sorting, filtering and selection warrant its headless table model. Decide
  separately whether spreadsheet-style keyboard editing/paste is truly needed;
  do not assume a table library provides it.
- Defer charts until a report needs them. Consider **Recharts** only after the
  intended financial question, accessible alternative and estimate/actual
  semantics are clear.
- **Vitest + Testing Library**, **Playwright**, and automated accessibility
  checks (such as axe-core), with manual keyboard and assistive-technology
  review for critical workflows.

### Why this is a reasonable provisional choice

React/TypeScript is a familiar, broadly supported contributor/agent environment
with an extensive ecosystem. Strict types and explicit component boundaries aid
review, while Vite and CSS keep the frontend deployable as static assets. The
proposed libraries cover genuinely stateful forms and server interactions
without requiring a global state architecture. This does **not** establish that
React is uniquely best or that headless primitives minimize total code. The
counter-review finds no evidence in the product docs that establishes a better
framework, but says the component/grid strategy must be tested before adoption.

Keep amounts and rates as decimal strings at API boundaries. Do not use
JavaScript `number` for authoritative financial calculations. Support both
date-only transactions and transactions with a known or user-entered time.
Preserve the business date separately from the optional time/instant: a date-only
business date must not be silently shifted by the viewer's timezone, and a
source-provided time must not be discarded. If a time and timezone/offset are
provided, preserve that context for chronological ordering; do not invent a
time when only a date is known. The exact same-day ordering policy for entries
without time remains to define. Client previews must be labeled as provisional
unless supplied/confirmed by the server.

### Frontend counter-review and response

The strongest objections are that headless controls and plain CSS can transfer
work into application code; TanStack Table does not deliver spreadsheet editing;
generated/client types do not guarantee runtime or financial correctness; and
query refetches can conflict with unsaved review drafts. Accessibility gaps in
the prototype include dialog focus handling, labels, narrow-screen navigation,
and chart alternatives. Offline behavior is not specified and must not be
assumed.

**Revision:** keep the framework recommendation provisional, but prototype the
real editing workflows before choosing table/grid and component-library depth.
Keep form drafts independent of query cache, define stale-update/conflict
behavior with the API, and explicitly decide offline scope. Do not choose a
charting package yet.

### UX questions absent or under-specified in `explore/`

1. **Budget assignment editing:** cell, row editor or form; save/cancel, dirty
   state, keyboard navigation and conflict behavior. Partial and over-assigned
   budgets are intentionally saveable (AC-5.3–5.4).
2. **Import review:** account/worksheet selection; raw and interpreted columns;
   file-wide date/number/sign interpretation; editable rows; selection scope;
   mixed valid/invalid results; individual duplicate keep/link/discard and
   retained source history (AC-2.1–2.2, AC-2.9–2.10).
3. **Bulk action semantics:** whether selection includes filtered/paginated rows,
   how excluded duplicates are explained, and how partial failures/retries work.
4. **Split and FX allocation editing:** remaining amount, authoritative total,
   rounding remainder display, stable review ordering and manual edits that
   preserve totals (AC-3.2, AC-3.10, AC-3.12).
5. **Warnings versus blockers:** visible over-assignment, uncovered card debt and
   foreign-debt coverage warnings must not block valid actual activity; unresolved
   import ambiguity still requires resolution (AC-5.12, AC-5.14–5.17).
6. **Corrections and evidence:** show original source values, subsequent
   interpretations, conversion observations, review decisions and closed-period
   correction effects without implying the historical snapshot changed.
7. **Keyboard, focus and motion:** modal focus entry/return, Escape, error
   announcement, row removal after confirmation, restrained transitions and
   `prefers-reduced-motion` behavior.
8. **Responsive and localized dense workflows:** define narrow-screen behavior
   for import comparison and split editing; clarify Spanish/Chilean locale and
   distinguish currencies beyond an ambiguous `$` symbol.
9. **Period context:** reports and budgets can use different period bases. Decide
   whether the prototype's global month selector is appropriate or misleading.
10. **Planning displays:** separate pool holdings, expected payment sources,
    budget assignments, goal progress, shared affordability and scenario
    estimates; do not make charts the only explanation.

### Frontend validation gate

Build small prototypes for (a) manual transaction entry with a split and (b) an
import with mixed valid/invalid rows and a suspected duplicate. Include keyboard
only, narrow viewport, a failed/stale save, and a rounding example. Use those to
compare native forms/semantic tables against any richer component or grid option.

## Backend proposal

### Accepted starting architecture

- Retain **Rust**, as required by the accepted rewrite direction. Use a single
  **modular monolith** organized around product capabilities (accounts, ledger,
  imports, budgets, projections), with domain calculations independent of HTTP
  and SQL. Begin with logical modules; split crates only when there is a clear
  dependency or ownership benefit. Keep four responsibilities distinct:
  domain rules and money types, application use cases and transaction
  boundaries, SQLx persistence, and the HTTP adapter. Detailed repository
  contracts, connection configuration and migration layout remain open.
- **Axum on Tokio is selected** for the thin browser HTTP adapter. Keep domain
  calculations ordinary, readable Rust code; isolate blocking file
  parsing/projection work so it does not block async request handling. See
  `desc/10-backend-architecture.md` for the selected backend stack and coding
  conventions.
- Use **HTTP/JSON** for browser access. Keep the later CLI's transport
  undecided until its command and trust contract is planned. Prefer explicit
  command endpoints for meaningful user decisions such as import confirmation
  and period close. An OpenAPI contract and generated TypeScript transport
  types remain candidates pending a representative generation/validation
  spike.
- The accepted first-slice decision is **PostgreSQL**, using **SQLx with
  explicit SQL and versioned migrations**. Validate the schema, exact-money
  representation, actual queries and transaction semantics on PostgreSQL; use
  use-case-level transaction boundaries. For self-hosting, apply pending
  migrations at backend startup with SQLx migration locking enabled; hosted
  releases can invoke the same migrator as a separate release step. See the
  [SQLx migrator documentation](https://docs.rs/sqlx/latest/sqlx/migrate/struct.Migrator.html).
  SQLite is deferred as a possible
  later local desktop option, so cross-database parity is not a first-slice
  requirement. The accepted self-hosted reference runs separate frontend and
  backend containers with PostgreSQL; the future hosted reference uses a
  frontend deployment and independently scalable backend instances with
  PostgreSQL. Operator-specific provisioning and hosting details remain open.
- Use exact decimal-bearing domain types with explicit currency and checked
  arithmetic. `rust_decimal::Decimal` is accepted for first-slice calculations;
  its 96-bit mantissa and scale from 0 through 28 define the envelope. Store
   normalized values as PostgreSQL `NUMERIC`, validate exact parsing and
   round-trips, and reject overflow without silently changing a value. Derived
   CLP foreign-asset valuation rounding is accepted; other required division
   and rounding rules remain open. If SQLite is added later, validate its
  representation separately; do not assume PostgreSQL `NUMERIC` semantics
  carry over to SQLite. JSON monetary amounts and rates use decimal strings,
  not floating-point JSON numbers; this does not imply unlimited precision.
- Preserve source records, links, review decisions, ledger activity and
  correction history as distinct concepts. A financial decision and its audit
  record should commit atomically. Do not introduce event sourcing solely for
  auditability.
- Start without a general queue. Add narrowly scoped, resumable import work only
  if real file sizes/latency/recovery needs warrant it. If retried, financial
  effects must be idempotent and transactionally protected.
- Authentication mechanism, bootstrap, credentials, recovery, raw-file storage
  and encryption remain explicit decisions. Keep initial data scoped to the
  owner/workspace boundary; separate household-member identities are later
  scope. The authentication mechanism for self-hosted and hosted operation is
  still open.

### Why retain Rust, and the tradeoff

Rust is already the agreed rewrite direction (`desc/00-rewrite-context.md`,
“Rewrite direction”), and can support explicit financial types, safe service
boundaries and a self-hostable binary. A strong compiler helps catch structural
mistakes but cannot prove a budget rule correct. Rust can also increase compile,
boilerplate and async integration cost. A Rust spike should test a transfer, a
reviewed import and a closed-period correction—not just CRUD. Reopening the
language decision would require an explicit owner decision; TypeScript,
Kotlin or C# are credible alternatives only if team delivery evidence justifies
that change.

### Backend counter-review and response

The earlier counter-review supports Rust provisionally and challenges treating
Axum as the fastest route; recommends proving rather than assuming OpenAPI generation;
flags PostgreSQL's meaningful self-hosting tax; notes that SQLx does not define
atomicity or concurrency; highlights bounded decimal precision and division
rounding risks; and finds that a durable job table quietly creates queue
operations work.

**Current decision:** the owner delegated backend technical choices with
readability for a Rust beginner as the main constraint. Axum/Tokio, SQLx,
PostgreSQL, the bounded `rust_decimal` envelope, and the initial migration
workflow are selected; see `desc/10-backend-architecture.md`. Defer a generalized
worker and client-generation pipeline. Validate migrations, worked financial
cases, exact-value round trips, and transaction/concurrency behavior on
PostgreSQL. Reassess SQLite if a local desktop mode is pursued. Detailed
transaction, revision, uniqueness and retry contracts remain implementation
work.

### API and consistency principles to validate

- Preserve business dates as `YYYY-MM-DD`, separately from transaction time
  when available and audit instants (UTC timestamps). Support date-only
  transactions and timed transactions. Do not derive/shift the business date
  through the viewer's timezone; preserve source time/zone or offset when
  supplied. Do not manufacture a time for date-only data. Exact same-day order
  where time is absent, and interpretation when a source supplies a time without
  timezone/offset, remain to define.
- Amounts and rates cross JSON as validated decimal strings, with explicit
  currencies and provenance/status where relevant.
- Errors are structured and actionable; report filtering and pagination are
  explicit. Generated types do not replace runtime validation or authorization.
- Corrections, import confirmation and budget close need explicit stale-write
  handling. For first-slice mutations, the owner accepted a persisted
  idempotency key, request-hash and result contract; editable-record revisions
  and later correction workflows still need their own rules.
- Keep all records for a financial decision, its provenance and audit decision
  in one transaction. Test concurrent confirmation/correction and workspace
  isolation.
- Workspace-scoped data and authorization are required from initial schema and
  service boundaries. This does not decide membership or SaaS collaboration.
- A future CLI is an intentional product interface, but the CLI's timing,
  transport, credential model, write safeguards and compatibility policy remain
  open (`desc/00-rewrite-context.md`). MCP remains
  deferred.

## Frontend framework reassessment

The original React-first proposal did not compare frameworks or run workflow
prototypes. The owner raised concerns about React's render/hooks reactivity,
maintenance weight and preference for scoped styles. This section supersedes
the React-first recommendation above, while preserving its framework-neutral
API, money/date, draft ownership and accessibility principles.

**Owner-selected starting stack: Vue 3 + Composition API + strict TypeScript +
Vite, with Vue SFCs and scoped CSS.** Start with semantic HTML, existing CSS
tokens, and app-owned components. Do not adopt Tailwind or a general UI library
by default. Add focused, accessible primitives only for interactions that need
them. Consider a form library, server-state library or table model only after a
workflow establishes the need; defer charts and spreadsheet grids. The owner
later confirmed Vue and basic native controls in `desc/00-rewrite-context.md`;
the experiments in `desc/11-frontend-architecture.md` are optional evidence,
not gates to the selected stack.

Vue is preferred for validation because its dependency-tracked reactivity and
SFC locality align with the owner's preferences while retaining a mature app
ecosystem. This is not proof it is fastest or smallest: ref/proxy behavior,
watchers and composable boundaries can themselves obscure state ownership.
Server state, editable drafts, stale writes and authoritative financial
decisions still require explicit boundaries. React remains a valid fallback if
a proven control integration materially reduces custom work; Tailwind is
optional in React, and CSS Modules/plain CSS are alternatives.

Desk-reviewed alternatives (not measured):

| Candidate | Why consider it | Main caveat |
| --- | --- | --- |
| Vue 3 | Tracked reactivity, SFCs/scoped CSS and broad application ecosystem | Ref/proxy semantics, watchers and shared-state ownership need conventions |
| Svelte 5 | Concise components, runes and scoped styles | Compiler/rune/effect semantics and a smaller specialist ecosystem; validate integrations |
| SolidJS | Fine-grained signals directly match the reactivity preference | Smaller ecosystem and reactive ownership rules; JSX does not provide SFC styles |
| React | Broad ecosystem and contributor/agent familiarity | Render/hooks/effect model is an owner-identified friction; CSS Modules work and Tailwind is not required |
| Preact | Smaller core with familiar JSX | Hooks can retain React-like concerns; compatibility with selected libraries must be tested |
| Vanilla TypeScript | No framework runtime, browser-native semantics | Routing, DOM synchronization, lifecycle, draft retention and accessible interactions become app-owned framework work |
| Server-rendered HTML + selective enhancement | Native forms and explicit submits can keep simpler workflows small | Rich multi-row editing may require substantial enhancement; viable if simpler interactions are intentional |

The static exploration is native HTML/CSS/JS, but its navigation and actions
are mock behavior, not a no-framework workflow evaluation. The old PoC used
Vue, but its implementation guidance is explicitly historical and is not a
decision to reuse Vue. “Heaviest” must be measured as a defined metric—such as
compressed route assets, parsed/executed code, memory or maintenance cost—not
inferred from dependency installation size. No Lukas bundle or interaction
benchmark exists yet.

**Optional comparison:** time-box Vue versus Svelte using identical fixtures, API
stub, styles, browser support and acceptance checks. Include import review
(raw/interpreted values, editable invalid row, selection, duplicate exclusion,
failed/stale save), budget editing (dirty draft survives refetch; warning does
not block saving), a split display (CLP 1,000 → 334 + 333 + 333), keyboard-only
and narrow-screen use, focus restoration and an ambiguous timeout. Have a human
and an agent perform the same refactor in each: extract editor state, add a
review field and change a mutation response. Compare repair/refactor effort,
context needed, type feedback, accessibility findings, compressed initial/route
assets and latency at agreed row counts. Optionally include a small
server-rendered/native baseline. Since the owner has selected Vue as the
starting direction, perform this comparison only if runtime or maintenance
concerns justify revisiting that choice.

### Routing and URL state (accepted direction)

Use **Vue Router**. Put navigable feature identity and shareable view context in
the URL, including filters, selected report period and report period basis where
applicable. Reports and budgets can have independent period semantics, so do
not imply a single global month selection controls both. Keep ephemeral UI state
(selection, focus, expanded rows) and unsaved drafts local; do not serialize
them into the URL by default.

### Forms and validation (accepted direction)

Start with native HTML form controls and Vue-local form state; do not add a
form or schema-validation library initially. Use browser constraints for basic
input feedback, small explicit workflow validation functions, and show backend
validation errors at the relevant fields. Frontend validation is usability
support; financial rules and calculations remain authoritative on the backend.
Revisit a library if nested/repeated forms such as import mapping or split
allocation demonstrate that the current approach is cumbersome.

### API client (accepted direction)

Use the browser's native `fetch` behind one small typed API client. Centralize
base URL, credentials, JSON and decimal/date serialization, structured errors
and request/correlation IDs there; call it from Pinia Colada queries and
mutations. Do not add Axios or generate a client until the backend API contract
is settled. Keep the wrapper small so generated types/client code can be
introduced later without coupling application components to a Rust framework.

### Locale and date/time formatting (accepted direction)

Default presentation to Chilean Spanish (`es-CL`) and the workspace timezone
(`America/Santiago`). Keep currency codes explicit where `$` could be
ambiguous. Use shared formatting helpers, not ad hoc component formatting.
Preserve decimal strings and do not convert authoritative amounts through
JavaScript `Number` for display. Render the preserved business date without
shifting it to the viewer's timezone; show transaction time and timezone
context when known. A display-timezone preference must not rewrite the recorded
business date.

### Frontend build and container (accepted initial direction)

Build the Vue application with Vite as static assets and package it in a
separate frontend Docker image/container. Do not require a Node application
server in production or Docker Compose for production. Supply the API base URL
through runtime configuration so the same frontend image can target different
backend locations. Configure client-side route fallback. The initial local
deployment uses a canonical loopback origin and same-origin `/api` through the
frontend container. Cross-origin API deployment and its reverse-proxy topology
remain open for later deployments.

### Frontend project organization (accepted direction)

Organize by product feature, with a small shared layer: `app/` for setup and
routing, `features/` for accounts/transactions/imports/budget/goals/reports,
and `shared/` for genuinely reused UI, formatting and API client code. Keep
feature-specific components/composables within their feature; promote code to
`shared/` only when reuse is real. Avoid generic layers and cross-feature
implementation imports that make agent/human navigation expensive.

### Frontend testing (accepted direction)

Use **Vitest** for unit tests, **Vue Testing Library** for component behavior,
and **Playwright** for end-to-end workflows. Add automated accessibility checks
where practical, with manual keyboard and assistive-technology testing for
import review and budget editing. Test observable user behavior rather than
component internals.

Make the application testable by AI-operated browsers as well as deterministic
automation: use semantic roles, accessible names and labels; provide stable
fixtures and a repeatable test-data setup/reset path; avoid interactions that
depend only on coordinates, color or canvas pixels. AI-browser testing is an
additional exploratory/acceptance path, not a replacement for repeatable
Playwright tests or human accessibility review. The specific AI browser/tool
integration remains open; this does not imply product MCP support.

The counter-review's key caution: Vue/Svelte reduce component ceremony, not
financial workflow complexity. Watchers/effects must not become an implicit
financial command bus; fetched server state, unsaved drafts, transient view
state and URL state need separate ownership. A vanilla client may merely
recreate a framework's lifecycle/state conventions, but server-rendered HTML
with modest enhancement remains credible if the product accepts row/detail
editing instead of spreadsheet behavior.

### Query, grid and chart libraries (revised requirements)

Spreadsheet-like amount editing and graphics are known requirements. The prior
recommendation to defer grids/charts was too broad: defer selecting a package,
not validating those capabilities. These shortlists are not accepted decisions.

**Server state:** Use **Pinia Colada** for query/cache behavior; TanStack Query
will not be used. Pinia alone is general client state, not a replacement for a
server-cache policy. Regardless of library, separate fetched snapshots, dirty
editor drafts, view state and URL state. A timeout is an unknown write outcome,
not proof the operation failed; do not blindly replay financial commands.

**Grid:** Start with simple semantic HTML tables and ordinary HTML inputs for
the current workflows. Move to a spreadsheet/grid library only when a concrete
scenario demonstrates that the simpler interaction is insufficient. At that
point, compare AG Grid Community (check feature tier if range selection or
spreadsheet clipboard behavior is required) and Handsontable (review
commercial/noncommercial terms and accessibility). TanStack Table is a
headless table model, not a spreadsheet editor. Do not pay the grid's bundle,
interaction and licensing costs before the need is demonstrated.

The requested `+ - * /` amount entry does not imply a workbook formula engine.
Treat it as a small, bounded expression language: preserve raw text, parse
without `eval`, define locale separators, precedence, parentheses, division
precision/rounding, overflow and error feedback, then let the server validate
and calculate authoritative results. Keep amounts as decimal strings; never
round each allocation cell independently. The agreed CLP 1,000 equal split is
334 + 333 + 333 with a visible, stable remainder assignment (AC-3.12). Whether
the grid needs range paste/fill, cross-cell references, undo-after-save or
batch commit remains open. Check licensing for self-hosted distribution and
possible future SaaS before adopting commercial-tier features.

**Charts:** Graphics are required, so select at least one representative chart
library during the frontend prototype. Chart.js with its Vue wrapper is a
candidate for conventional category bars and cashflow lines; Apache ECharts
with its Vue wrapper is a candidate if richer interactions/chart types justify
its extra surface. A small app-owned SVG chart can be compared for simple needs.
Measure production route assets and interaction behavior rather than package
installation size. Exact totals remain computed outside chart coordinates;
any numeric conversion for plotting is display-only. Every chart needs an
accessible exact-value table or equivalent, a textual summary, and clear
actual/pending/projected and period-basis distinctions. Do not imply historical
net-worth trends are settled; the valuation rule remains open.

Start with semantic HTML tables/inputs; revisit AG Grid or another editor only
when an identified workflow requires it. Compare Chart.js vs ECharts (or SVG)
on a spending breakdown and actual-versus-projected series before the first
chart implementation. Include accessibility checks, exact-value alternatives,
bundle measurements and human/agent refactor effort. Product decisions still
needed include cell/row/batch commit; range paste/formulas; realistic row
counts; and chart questions/filters.

Charting direction accepted: graphics are required, but the package choice is
deferred until the first chart slice. Begin with conventional charts, compare
Chart.js with ECharts only if required interactions warrant it, and preserve
accessible exact-value alternatives and server-authoritative calculations.

### State ownership conventions (accepted direction)

Keep five kinds of state distinct: (1) authoritative financial facts and
calculations live on the backend; (2) fetched snapshots/request status belong to
Pinia Colada; (3) unsaved edits live in local component/composable drafts with
their baseline revision; (4) transient selection/focus/expanded-row state stays
view-local; and (5) shareable filters, period and report basis belong in the
URL. Do not bind editable fields directly to cached server objects or add
Pinia/global state solely to duplicate query data.

Refetches must not overwrite dirty drafts. On revision conflict, retain the
draft and offer explicit reload/review behavior. Use explicit commands for
consequential decisions; after successful mutations, initially invalidate
affected server queries rather than manually patching dependent financial
totals. Do not optimistically present calculated money as confirmed. A
timed-out write has an unknown outcome; an explicit retry reuses the accepted
idempotency key and exact request. Backend revision and conflict contracts
remain to be defined. See `desc/00-rewrite-context.md` and `desc/07-domain-model.md` §1 for the
financial state distinctions.

## Styling and component conventions (accepted direction)

Use framework-independent CSS custom properties for
design tokens, CSS cascade layers for reset/base/components/utilities, and
Vue SFC `<style scoped>` for component-local styles. Keep global CSS small and
semantic; use ordinary CSS rather than Tailwind or a styling-in-JS system.
Start without a general component library: compose native semantic controls
into a small app-owned set of buttons, fields, alerts, tables, dialogs and
status badges as real workflows establish their needs. Use a focused accessible
primitive only where native HTML is insufficient.

This builds on `explore/base.css` (`@layer`, tokens, focus behavior) while
consolidating the separate inline token vocabulary and prototype classes in
`explore/design-system.html`. Retain the visual direction as a starting point,
not accepted branding or a finished design system. Avoid copying duplicated
prototype styles into components. Prefer self-hosted fonts or system fallbacks
over a runtime dependency on Google Fonts.

Rationale: native CSS is sufficient for the expected app size, readable to
humans and agents, and avoids adding a styling tool and its vocabulary. Some
repetition is acceptable; revisit only if it becomes a demonstrated maintenance
problem. SFC scoping does not provide accessibility, behavior or design
consistency automatically; global tokens and cross-component states still need
conventions. Retain the visual exploration as a starting point, not accepted
branding or a finished design system.

## Owner decisions and open questions

1. **Database and operations:** PostgreSQL with SQLx and versioned migrations
   is accepted for the first slice. Startup migrations are accepted for
   self-hosting; hosted deployments can run the same migrator separately.
   Define migration layout, connection configuration, upgrades, backup/restore,
   remaining numeric behavior and concurrency checks for PostgreSQL. SQLite is
   deferred; production provisioning/topology details remain open.
2. **Numeric behavior:** The first-slice calculation envelope is accepted as
   `rust_decimal::Decimal` (96-bit mantissa, scale 0–28), persisted as
   PostgreSQL `NUMERIC`, and is the only input/storage scale cap; preserve exact
   supplied values within it. Plain ASCII decimal-string syntax and the field
   sign rules are accepted. Derived CLP foreign-asset valuation rounds once to
   whole pesos, nearest with midpoint away from zero. Define other division and
   rounding behavior, overflow reporting and stable residual tie order. Never
   silently truncate source facts. Raw-source precision beyond the calculation
   envelope and unusual source-total precision remain open for import planning.
3. **API contract workflow:** Which schema is authoritative? Prove decimal/date
   behavior, uploads, nullable/union types, error shapes, and generated client
   quality before adding generation to every build.
4. **Concurrency and writes:** The first-slice idempotency contract is accepted:
   each mutation uses one UUID key and exact request per user intent; retries
   replay the saved result, changed payloads conflict, and minimal command
   metadata remains for the workspace lifetime. Define editable-record
   revisions, conditional update/locking rules, and bounded deadlock or
   serialization retries. Later duplicate confirmation and correction
   collisions remain to be designed for the slices that use them.
5. **Authentication and data protection:** Initial no-auth access is accepted
   as same-machine only, with a loopback-published frontend and private backend
   network. Before wider access, define self-host bootstrap, TLS, session/token
   model, recovery, CLI credentials, raw statement retention/encryption and
   backup scope.
6. **Background work:** Which concrete MVP operations need durable execution,
   and what file size, latency, retry and recovery requirements apply?
7. **Frontend interaction model:** Is spreadsheet-speed budget entry a real
   requirement? What list sizes, browsers, assistive technologies, locale and
   offline behavior are supported?
8. Continue resolving product-domain questions separately, including same-day
   opening history, positive card-credit use, goal-linked refunds, and closed
   period correction fallback. Technology choices must not silently settle
   those questions (`desc/07-domain-model.md` §9 and rewrite context).

## Suggested decision sequence

1. Integrated first vertical slice: connect the Vue client to the Rust API and
   PostgreSQL through the local Compose topology for account creation with an
   opening balance, manual activity, and current-month budget assignment.
2. Optional frontend experiments: compare Vue/Vapor/Svelte only if performance
   or maintenance concerns warrant it; compare native controls with PrimeVue
   before adopting a general UI suite.
3. Database validation: run fresh PostgreSQL migrations, exact-money round trips,
   worked cases and transaction/concurrency checks. Document the results.
   Assess SQLite compatibility later if desktop-local storage is pursued.
4. Numeric validation: test exact PostgreSQL round trips and checked arithmetic at
   the accepted envelope, including negative refunds and overflow. Define and
   test division/rounding behavior before features that require it.
5. API validation: prove date-only/decimal-string serialization, stale-write
   errors, budget assignment/activity contracts, and retry safety. Import
   upload and generated-client workflow belong to later planning.
6. Record accepted decisions and remaining open questions in focused ADRs before
   implementation expands.

## Agent review setup

The frontend and backend proposals were produced independently by
`openai/gpt-6.1-terra` and challenged independently by
`openai/gpt-6.1-sol`. The reviews are advisory and do not substitute for
maintainer familiarity, prototypes or owner acceptance.
