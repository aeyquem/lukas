# Rewrite context

Status: Accepted

This document captures the current product and domain context for the next
Lukas implementation. It records the rewrite direction agreed by the project
owner and the useful constraints found in the existing documentation. It is a
planning handoff, not an implementation specification. The existing application
and its framework-specific guidance describe the proof of concept and should
not be treated as requirements for the new implementation unless explicitly
carried forward here or reconsidered later.

## Rewrite direction

- Replace the Tauri desktop application and .NET sidecar with a Rust
  implementation and a web application.
- The owner delegated backend technical choices to the architecture lead, with
  readability for someone new to Rust as the main constraint. See
  `desc/10-backend-architecture.md` for selected choices and open items.
- Design the product as a self-hosted solution first, with a path to SaaS if
  that becomes useful. Keep product/domain behavior distinct from deployment
  and hosting concerns so hosted operation remains possible.
- Give the Rust backend and web frontend separate Dockerfiles so each can be
  deployed independently to Docker-compatible environments, including cloud
  platforms that do not use Compose. Use Docker Compose for local development.
  The accepted reference topology for now is one self-hosted machine running
  separate frontend and backend containers with PostgreSQL, developed together
  using Docker Compose. The future hosted reference uses a frontend deployment
  and independently scalable backend instances connected to PostgreSQL. SQLite
  is deferred as a possible later local desktop option. Operator-specific
  proxy/TLS configuration, PostgreSQL provisioning and storage, upgrades and
  backups, and the exact SaaS-compatible boundary remain undecided.
  The self-hosted backend applies versioned migrations at startup with SQLx
  locking enabled; hosted deployment can run the same migrator as a separate
  release step.
- The first slice uses PostgreSQL. Keep product rules independent of database
  and hosting-provider behavior so SQLite can be considered later for a local
  desktop mode without making it a first-slice requirement.
  Supabase Auth is not required by the owner-only product model; whether it is
  used for a hosted deployment remains open.
- The owner selected Vue 3, strict TypeScript, Vite, Vue Router, native HTML
  controls, a small native `fetch` API client, and Pinia Colada for frontend
  server-state caching. Support Firefox and Chrome, including Zen; the
  application requires a live API and has no offline mode. Begin with basic
  forms and semantic tables. The first working slice must connect the browser
  through the Rust API to PostgreSQL rather than relying on a permanent mock
  backend. See `desc/11-frontend-architecture.md` for state ownership and the
  optional framework performance experiment.
- The first version has no authentication. Keep a future path open for email
  and password registration, OpenID Connect providers such as Google or
  Microsoft, or both; no provider or future identity design is selected. The
  first unauthenticated deployment is reachable from the same machine only:
  publish the frontend on loopback, keep the backend on the private container
  network, and do not expose it through a LAN-facing or public proxy. Any wider
  access requires a separately reviewed access-control design.
- The initial self-hosted instance is owner-only. One owner can manage personal
  or household finances; separate member identities and shared access are not
  part of the first slice. Later multi-member support remains open.
- Deprecate MCP for now. Provide a CLI that both people and agents can use to
  query information and add data to the program.
- The CLI is an intentional product interface. Its commands, output formats,
  authentication/trust boundaries, write safeguards and compatibility policy
  still need definition. Do not assume the old deferred public API/MCP plan
  answers those questions.
- Detailed persistence schema, container operations, future authentication and
  CLI distribution remain open product/operations questions. Backend
  implementation choices are recorded in `desc/10-backend-architecture.md`.

## Product purpose and initial audience

Lukas is a personal-finance application, initially focused on people in Chile
and their local financial behavior. The original product concept aims to help
people manage personal or household finances, including local bank activity
where practical. The product should prioritize financial trust: imported data
is reviewable, corrections are visible, and reports explain whether values are
actual, pending, estimated, adjusted or reconciled.

The initial workflows documented for the PoC are manual account and transaction
management; CSV/Excel imports; monthly budgeting and cashflow; goals and
reserves; and basic projections. Multicurrency is part of the domain model from
the beginning, with CLP as the initial reporting base and Chilean date, money
and financial conventions as defaults. These are product context, not a frozen
feature roadmap for the rewrite.

## Product principles worth preserving

- **Chile first:** begin with CLP, local institutions, local formats and
  Chilean financial behavior.
- **Multicurrency in the model:** accounts and movements retain their actual
  currencies; reports use a workspace base currency.
- **Manual and reviewable workflows:** automation must not make unreviewed
  imports silently authoritative.
- **Integrations behind adapters:** the domain should not depend on a bank,
  aggregator or provider payload shape.
- **Auditable history:** preserve provenance, corrections, reconciliation and
  user decisions.
- **Explainable projections:** show the assumptions behind scenarios and
  results.
- **Self-hosting first:** users should be able to operate their own instance;
  later SaaS operation should not require rewriting product rules.
- **CLI as an agent and human interface:** queries and data entry should be
  usable without the graphical web app, with explicit, understandable
  behavior.

## Product capabilities documented in the PoC

### Ledger and reconciliation

- Create and edit income, expense, transfer, fee, refund and adjustment
  activity, with account, amount, currency, date, description, category and
  provenance.
- Categorize and tag transactions. Categories are budget-oriented; tags are
  workspace-managed cross-cutting labels with normalized matching.
- Represent transfers between owned accounts as one logical transfer with
  ledger entries, without counting them as income or expense.
- Represent cash as an account; distinguish withdrawals (transfers) from cash
  spending.
- Detect likely duplicate activity and let users merge, keep separate, ignore
  or discard candidates while retaining the evidence and decision history.
- Reconcile calculated balances with institution balances/statements without
  silently rewriting history to conceal mismatches.

### Imports and integrations

- Manual entry, CSV and Excel are the first source types in the existing
  product plan. Imports map to one account before row parsing; multi-account
  files are split by the user for the PoC.
- Repeated and overlapping imports are expected. Import processing should be
  idempotent and route duplicates, ambiguous rows and corrections through
  review.
- Preserve raw and normalized snapshots before creating or linking ledger
  transactions. Provider-specific data belongs in adapters/snapshots, not the
  core domain.
- External IDs are matching evidence, not permanent identity. Account for
  missing, reused or changed IDs, provider mutations, tombstones and discard
  decisions so later imports do not resurrect hidden records.
- Mock Open Finance Chile, real Open Finance providers, investment APIs such as
  Fintual, Splitwise and other aggregators were exploratory or deferred
  integrations, not current commitments.

### Budgets, periods and money

- Owned money transfers between two budgetable accounts, or between two
  tracking-only accounts, have no budget-boundary funding effect. Independent
  FX differences and exposed fees retain their existing treatment and are
  counted once. A transfer into the
  budget from a tracking-only account adds funding once; a transfer out removes
  funding once. Explain the budget effect separately from income/spending:
  the financial movement remains a transfer. Do not silently change category
  assignments or funded goals; reduced funding may leave an over-assignment
  warning. Preserve real dates and closed snapshots under existing correction
  rules. Initial signed balances follow the rule below; positive-credit
  consumption follows the rule below and returns retain transfer treatment.
  Purchase coverage follows
  the rule below.
- Preview budget funding effects before confirming an account's eligibility
  change. From confirmation onward, tracking-only to budgetable adds the
  current eligible positive asset balance once; the reverse removes it once. This is a
  prospective funding adjustment, not a physical transfer, income, expense,
  category use or goal usage. Keep assignments/funded goals fixed and expose
  any over-assignment. Do not change actual balances, original bank data, real
  transaction dates, independent categorization or closed budget snapshots.
  Do not replay original receipts or prior valuation adjustments. Existing
  foreign-asset valuation rules apply. Existing debt and coverage survive
  eligibility changes under the debt rule below.
- Making an indebted card or negative-balance account tracking-only does not
  erase already-recognized debt, release its protection/reservations or create
  ready-to-assign. Positive assets cease funding prospectively, while prior
  debt retains its coverage or uncovered status until settled under existing
  rules. Enabling budgetability recognizes only debt not already represented,
  once under opening-debt rules, without deducting it twice or inventing funded
  coverage. Payments of already-recognized debt retain their once-only
  treatment even after the target becomes tracking-only: covered portions
  consume protection and uncovered portions retain the existing paid-uncovered
  gap treatment, without another ordinary boundary-outflow funding deduction
  for either portion. Unrelated asset transfers still follow the boundary rule.
  Preview effects,
  preserve actual balances, sources, dates and closed snapshots, and keep
  over-assignment warnings rather than blocking valid activity. Existing FX,
  payment, refund and positive-credit rules remain in force.
- Monthly budget and cashflow views cover income, spending, transfers, goals,
  reserves and remaining money. Distinguish actual, pending, projected,
  adjusted and reconciled values when it matters to interpretation.
- Initial budget funding uses actual signed budgetable balances: negative
  bank/cash/savings balances reduce funding; an actual positive credit-card
  credit adds funding. Unused credit and overdraft limits contribute nothing.
  For an eligible foreign-currency opening liability, use a separate
  user-supplied signed CLP opening funding basis once; preserve the original
  foreign-currency balance and do not deduct daily debt FX. If that basis is
  missing, funding and ready-to-assign are unavailable.
  For example, CLP 100,000 in one bank, CLP -20,000 in another and an actual
  CLP 5,000 card credit provide CLP 85,000 before assignments. Preserve existing
  opening card-debt treatment and count later transaction, purchase and bill
  reservation effects once. A positive card credit is not automatically new
  income. Its consumption follows the rule below; eligibility
  continues to govern budget participation.
- Card purchases record full actual spending even when budget funding is
  short. Actual positive card credit is consumed before new debt, as below. A category can become negative; reserve only its genuinely
  cash-covered borrowing portion after positive credit use. Show unfunded
  borrowing as debt and unfunded use of existing credit as asset use, not
  another debt portion. Do not
  automatically sweep ready-to-assign or other categories. An explicit later
  funded assignment/reassignment can fill the gap once without more spending
  or debt. Over-assignment may be saved with a warning but cannot invent
  cash-covered reservations. Budget and coverage shortfalls are warnings, not
  hard limits on recording actual purchases/payments allowed by the bank.
  Lukas does not initiate or authorize banking activity; required fields and
  import ambiguity, duplicate review and confirmation still apply. For example,
  a CLP 1,000 purchase with CLP 600 available produces CLP -400 category
  availability, CLP 600 reserved and CLP 400 uncovered debt. An explicit
  genuinely funded CLP 400 assignment leaves zero category availability and
  CLP 1,000 reserved, with spending/debt unchanged at CLP 1,000. Preserve
  opening-debt, once-only, FX and fixed-funding policies. Linked refund
  coverage and positive-credit consumption follow the rules below. Ordinary
  goal-linked refunds follow the rule below; exceptional/excess cases remain open.
- A bill reservation commits the same budgeted cash, usually held at a bank,
  to a later card payment; it is not a second assignment, a separate account
  or money held on the card. For same-currency payments between budgetable
  accounts, consume existing payment coverage and keep unused coverage
  reserved. Existing opening debt already deducted from initial funding has
  protection and needs no second funding. Payment releases no ready-to-assign
  money and creates no spending. A payment beyond coverage remains recordable:
  distinguish its debt-repayment portion paid without coverage from still-unpaid
  uncovered debt, without adding either on top of the original gap. Later explicit
  funding covers paid portions without reserving them again, reserving only
  still-outstanding debt. Do not silently sweep other categories. For example,
  debt 1,000/reserve 600/gap 400 followed by payment 500 leaves debt 500,
  reserve 100 and gap 400; payment 800 instead leaves debt 200, reserve zero,
  paid-uncovered 200 and unpaid-uncovered 200. Spending/category overspending
  do not change. Coverage-first is budget tracking, not the bank's payment
  allocation; statement reminders follow actual evidence. Preserve real dates
  and closed snapshots. Refund coverage and ordinary goal-usage reversal follow
  the rules below; exceptional/excess goal refunds remain open. Positive-credit
  consumption follows the rule below.
- A refund linked by source/user evidence to an underfunded card purchase
  repairs its uncovered amount first, then releases corresponding unpaid bill
  coverage. Reduce the original purchase's category spending once and restore
  that category's availability without income or new ready-to-assign. An
  unlinked refund retains user-selected categorization; do not apply linked
  purchase coverage restoration to a different category. A CLP 1,000 purchase
  funded with CLP 600 has CLP -400 category availability/reservation 600/debt
  1,000. A refund 300 leaves category -100/reservation 600/debt 700/uncovered
  100. A refund 500 instead leaves category 100/reservation 500/debt 500 and
  no uncovered debt. For fully funded purchases, refunds before payment
  release matching unpaid coverage; after full payment, the actual refund
  may create positive card credit and restore category availability, without
  releasing already-consumed reservation again. Preserve actual refund dates,
  source/user links, card balances and closed-period correction rules. Do not
  infer bank application to unrelated purchases or move money. Ordinary
  goal-linked refunds reverse usage under the goal rule below; exceptional/excess
  goal refunds remain open. Spending positive credit follows the rule below,
  and returns of owned money retain transfer treatment.
- Purchases use a real positive card credit first within the affected currency
  balance. Record the full category/linked-goal purchase once; only the new
  borrowing can need bill reservation, and only genuinely funded coverage is
  reserved. No income or extra ready-to-assign arises from credit already
  counted in opening funding or refund category restoration. Residual credit
  stays an asset; different currency balances are not offset without actual
  conversion/application evidence. For bank 700 plus card credit 300, funding
  1,000 supports a 500 Groceries assignment and 500 ready-to-assign. Buying
  500 leaves bank 700, debt/reservation 200, Groceries zero and ready-to-assign
  500. Underfunded spending still warns without fictitious coverage or duplicate
  gaps. A payment exceeding actual debt may create positive card credit: its
  non-debt portion is a transfer into an owned asset, not another expense,
  income or paid-uncovered debt. Preserve existing transaction and review rules.
- Accounts have a base currency and retain real separately denominated balances;
  a physical bank or card account can contain multiple currency wallets.
  Workspace reporting starts with CLP, independently of an account's base
  currency. Budget eligibility and pool grouping remain account-level choices.
  Retain each original purchase and the actual bank conversion chain, with
  amounts, currencies, dates and sources; a reporting estimate is not evidence
  that the bank converted a balance.
- MVP budget assignments, goal targets and contribution plans use CLP;
  foreign budget/goal-target currencies are later scope. Foreign planned cost
  components can retain their original amounts with CLP estimates. A selected
  current rate, optional user spread or available card-network rate can inform
  locked/floating estimates, without committing to an integration.
- A floating cost estimate changes forecasts/estimated funding gaps; after
  purchase it does not automatically revalue recorded spending, category/goal
  remaining amounts, funding or reservations. Actual debt stays in its real
  currency, independently valued at current rates for net worth. Locks remain
  provisional; confirmed bank conversions/reviewed derived allocations update
  the original expense difference once under existing correction rules and
  never create automatic goal funding. Significant-change alert thresholds
  remain open. Eligible foreign assets have a separate automatic budget
  revaluation policy; cost-estimate settings do not control that policy.
- In the first manual-entry slice, a foreign-currency expense without a
  confirmed CLP statement value must retain its original amount and must not
  receive an invented transaction-date estimate. Represent affected CLP
  activity and category availability as incomplete, retain known CLP subtotals
  and list unresolved original amounts, and complete the existing expense once
  a confirmed value is supplied. Dependent aggregate availability is
  incomplete. Ready-to-assign can be complete only if its own funding inputs
  are complete; show an adjacent pending-expense warning. Assignments remain
  savable intent even when coverage is unknown. Do not close a period while
  required values are incomplete; never silently count the expense as zero.
  This is accepted MVP behavior. See `desc/08-first-slice-plan.md`.
- For an eligible foreign-currency opening liability, preserve the signed
  original-currency debt and require a separately supplied signed CLP opening
  funding basis. Apply that basis once to initial funding; do not use daily
  debt FX for further funding deductions. If the basis is missing, funding
  and ready-to-assign are unavailable. When the foreign opening-liability
  amount is directly corrected, the user must supply or explicitly confirm
  its signed CLP funding basis in the same update; never derive the revised
  basis from daily debt FX. This paired-update rule is accepted. Basis storage
  and provenance, and how apply/offset corrections affect it, remain to
  specify.
- MVP budget funding automatically reflects daily mid-market CLP changes in
  eligible foreign money held (assets), via explainable valuation adjustments
  distinct from income/spending. Positive differences add ready-to-assign
  once; decreases can expose over-assignment without altering assignments,
  funded goals or targets automatically. Closed snapshots remain intact;
  current effects follow existing correction rules. Tracking-only accounts
  still show current valuations but contribute no budget funding. Actual
  incoming conversions retain bank amounts; a foreign-balance estimate does
  not invent conversion. Receipts/transfers and previous valuation changes
  are never counted again. Expense-cost estimates remain warning-only and
  do not automatically revalue usage/remaining amounts; foreign debt follows
  the warning-only coverage rule below. User-selectable automatic versus reviewed
  asset revaluation is later scope, with automatic behavior in the MVP.
  Frankfurter v2 is the selected daily mid-market rate source. On weekends,
  holidays, or a provider outage, use the latest stored observation and show
  its effective date with a stale label when the provider is unreachable. If
  no observation is stored, mark foreign valuation unavailable; do not treat it
  as zero or return a partial budget-funding total.
- Outstanding foreign card debt retains its real denomination until paid or
  actually converted. Current FX changes its CLP net-worth valuation and
  estimated coverage warning only, not recorded purchase valuation, genuine
  reservation, fixed assignments or ready-to-assign. Show that warning alongside
  ready-to-assign: available assignment money alone does not guarantee current
  debt coverage. Keep eligible foreign-asset auto-revaluation separate. Combine
  original uncovered debt and incremental FX difference into one explained
  shortfall; never add the whole estimate on top of the original gap. Revalue
  only outstanding foreign portions, not settled or fixed converted CLP debt.
  Favorable FX does not automatically release coverage. Actual conversion or
  confirmed allocation replaces the provisional difference with the actual
  difference once, following source and closed-period correction rules, without
  another purchase or automatic funding. Opening-debt conversion differences
  adjust existing protection/funding once and create no fresh spending. For
  example, USD 100 initially valued/reserved at CLP 90,000 is worth CLP 95,000
  at a later 950 rate: show a CLP 5,000 estimated gap while reservation and
  ready-to-assign remain unchanged. Actual CLP 95,000 conversion applies that
  CLP 5,000 expense difference once; explicit funded coverage can then fill it.
- Do not use floating-point arithmetic for monetary amounts or rates. The PoC
  proposed decimal domain values and decimal-text SQLite persistence. For the
  PostgreSQL first slice, `rust_decimal::Decimal` is accepted as the bounded
  exact calculation type, with PostgreSQL `NUMERIC` storage. Its 96-bit
  mantissa and scale range of 0–28 define the first-slice calculation envelope;
  this is the only input/storage scale cap. Do not impose currency-specific
  scale limits or round a valid value merely to match display precision;
  ordinary field validation and the Decimal envelope still apply. Preserve
  exact values for calculations and inspection while presenting ordinary CLP
  in whole pesos and USD in cents.
  accept plain ASCII decimal strings with digits, an optional leading minus
  and decimal point; reject exponent notation, grouping separators,
  whitespace, NaN and infinity. Opening balances and foreign-liability CLP
  bases are signed; transaction and transfer request amounts are positive
  magnitudes whose kind/direction determines signed ledger effects; category
  assignments are nonnegative, while category remaining may be negative.
  Parse or arithmetic overflow must be handled without silent rounding or
  truncation. For derived CLP foreign-asset valuation, multiply the exact
  foreign amount by the exact rate and round once to a whole peso using nearest
  with midpoint away from zero. Other division/rounding policies,
  application/repository boundaries, and connection configuration remain
  open. If SQLite is added later, validate its migration and query needs then.
- A reporting conversion is an observation attached to financial activity,
  not one mutable converted amount. Actual bank conversion can also change
  which currency holds funds or debt on its real conversion date; it does not
  create another expense or duplicate the old and new balances. Relevant
  observations include institution pending estimates, final billed/paid amounts,
  manual corrections, reconciled values,
  provider corrections and fallback historical estimates. Reports choose a
  contextually appropriate observation; stored historical estimates support
  repeatable reports.
- Bank-supplied purchase conversion amounts take precedence. When only an
  aggregate currency conversion is supplied, preserve it as authoritative
  account evidence and label internal purchase/category/goal allocations as
  derived. With known composition, suggest proportional allocation only across
  eligible outstanding converted portions; require review/confirmation and
  allow manual changes. Partial conversions preserve settled portions and
  follow the same rule. Allocations reconcile to the bank total with rounding
  discrepancies visible. Incomplete composition can leave purchase valuation
  provisional without preventing recording the actual aggregate conversion.
  Preserve the meaning of purchases, refunds, exposed fees, opening debt and
  payments; conversions create no new expense, debt duplicate or goal funding.
- Ordinary entry/display precision uses whole CLP pesos and USD cents, while
  retaining higher supplied precision and rates for inspection. Calculate
  derived splits/conversion allocations before rounding and make rounded
  portions sum exactly to the authoritative total. Distribute residual units
  by largest fractional remainder, using displayed review order for ties;
  show the adjustment and allow valid manual changes retaining the total.
  Preserve bank-supplied purchase amounts and refund signs; never create an
  extra expense for rounding. Unusual source-total precision remains an edge
  case to refine without truncating the supplied amount.
- Keep fees, spreads, taxes and card charges as separate entries only when the
  source exposes them; do not invent detail from a final charged amount.
- Model business dates as dates, not implicit local timestamps; store audit and
  technical timestamps as UTC instants. The PoC default workspace timezone was
  `America/Santiago`.
- A transaction can have calendar, budget, statement, cashflow or future tax
  period context. Assigning a period must not rewrite its real transaction
  date. Calendar months are the first budget view; source-specific rules may
  be needed for statements, salary timing and tax.
- For manual entries, if the transaction date is before its wallet's opening
  date, ask whether to apply its effect to the current balance or preserve the
  current balance by offsetting the wallet's opening amount. If it equals the
  opening date and inclusion is unknown, require the same initially unset
  choice per affected wallet. Do not infer chronology from date, ID or record
  order.
- Budget periods remain open until the user explicitly closes them; calendar
  month-end never closes a period automatically. At explicit close, preserve
  the closed snapshot and carry existing unassigned ready-to-assign forward;
  do not reseed the same funds from account balances. For each category, the
  user chooses whether its signed remainder returns to ready-to-assign or
  carries in that category, as in AC-5.5. This choice is in the MVP. Preserve
  the sign: returning a negative remainder reduces ready-to-assign, while
  carrying it leaves a negative amount in that category. Period close is
  blocked while required values are incomplete. Other transition details
  remain open. While
  open, transactions can be added or edited even after the
  calendar period ends. The next calendar month's budget accepts assignments
  even if the preceding month remains open. Calculate each month from the most
  recent explicit close plus the intervening activity; an unclosed month is
  still calculable and can receive assignments. When a month is later closed,
  its computed values become the checkpoint for subsequent calculations. During
  provisional calculations, carry each signed category remainder in the same
  category into the next month. At close, the user may keep that carry or
  release it to ready-to-assign per category. Recalculate later months from the
  resulting close without counting activity twice or changing assignments.
  This timing is an MVP rule. For example, closing November during December
  replaces the earlier rolling
  calculation from the prior close with a calculation from November's close,
  without counting intervening activity twice or changing assignments already
  made in December. This does not auto-close the preceding month. This basic
  close transition is in the first slice;
  advanced multi-period rollover, goals and pools remain outside it.
  The first calculation starts from zero, then adds eligible signed opening
  balances once. Later calculations roll from the latest explicit close plus
  intervening activity.
  Closed periods may still be corrected by editing the transaction directly;
  retain its actual date and audit history, preserve the closed budget
  snapshot, and apply the budget impact in the current period where possible.
  There is no period lock.

### Goals, scenarios and reports

- Goals and reserves represent allocations, not hidden accounts or unexplained
  ledger movements.
- An ordinary linked goal-purchase refund reverses goal usage once and
  restores the original budget category once under the card/refund rules.
  Saved/funded, target and funded progress stay unchanged. For saved 5,000,
  used 1,000 and available 4,000, a linked refund 300 leaves saved 5,000,
  used 700 and available 4,300. A shared-category goal with usage 1,000 and
  shared availability 4,000 instead gets usage 700/shared availability 4,300,
  without individual funding or another restoration. Preserve original links,
  real dates, CLP valuations and closed-period correction rules; prior bill
  payment does not change the goal reversal. Exceptional saved-reduction
  refunds must refer to their original effect rather than new contributions;
  exact restoration and amounts exceeding remaining linked usage/FX cases
  remain open, along with corrected goal links. Explicit funding reassignment
  follows the rule below.
- Reassigning funding away from a specifically funded goal reduces net
  saved/funded, leaves used unchanged and recalculates available. Preserve
  contributions and explicit reassignment history; it is not account movement,
  income, expense, new funding or an exceptional saved-reduction transaction.
  Direct reassignment increases the receiving assignment once without also
  consuming ready-to-assign. Returning funding to ready-to-assign releases it
  once; later assigning it consumes it once. Shared-category reductions change
  candidate affordability once without individual saved amounts. Allow an
  excessive reassignment with a warning: funded 5,000/used 1,000/available 4,000
  minus 500 reassigned gives 4,500/1,000/3,500; minus 4,500 instead gives
  500/1,000/-500. This is one 500 coverage shortfall, not two source/recipient
  deficits, and cannot create actual money or genuine bill coverage. Actual
  balances and existing closed-period rules remain intact. Corrected links
  and exceptional saved-reduction/refund policies remain separate questions.
- The explicit manual goal link belongs to a purchase portion, separate
  from tags. Each portion has at most one goal; an unsplit purchase has one
  portion. Ordinary/shared-category links are optional; a specifically funded
  goal category mandates that matching goal association/effect, with neither
  an absent nor conflicting link. Selecting the category supplies that manual
  association once. Split portions reconcile to the parent total, allowing multiple
  goals without duplicating the account effect, expense or usage. A 300,000
  marketplace purchase can contain 220,000 Vacations/suitcase usage and 80,000
  Photography/camera usage, while total account effect and spending remain
  300,000. Specifically funded portions use their goal category. Shared-goal
  portions use a shared category with their own goal links, consuming the
  shared category once without individual saved amounts. Preserve existing
  per-portion goal funding, refund and reassignment effects; allocation of
  refunds across portions and corrected links remain open. Unlinked portions
  remain possible for ordinary/shared-category portions; automatic goal-link
  suggestions remain later scope.
- Projections may use historical activity, recurring patterns, current
  balances and user assumptions such as expected income, spending, one-time
  purchases, savings or debt payments.
- Estimates remain separate from actual ledger history. Scenarios compare
  alternative plans without mutating real history and expose their
  assumptions.
- A negative-income/survival scenario may include reduced spending, severance,
  liquidity, debt strategy, reserves and account exhaustion; it is a planning
  tool, not a guaranteed prediction.
- Initial report ideas: monthly budget/cashflow, spending by category/tag,
  account balances, goals/reserves, import/review status and basic projection
  or runway.
- Chile annual tax preparation was explicitly future scope or a possible
  adjacent module. It should preserve source-document traceability and should
  not be presented as official tax advice.

## Accepted backend structure

The accepted Rust structure is a single modular monolith with logical modules
for domain rules and money types, application use cases and transaction
boundaries, SQLx persistence, and a thin HTTP adapter. Keep domain behavior
independent of HTTP and SQL; do not split into multiple crates initially.
Detailed repository contracts, module ownership, and connection configuration
remain open.

The core financial distinction is between source facts and user-facing ledger
activity:

- **Source records** preserve facts from manual entry, files or future
  providers; their snapshots hold original and normalized interpretations.
- **Ledger transactions** represent normalized financial movements used by
  balances, reports, goals and projections.
- **Source links** connect a logical ledger transaction to one or more source
  records.
- **Import batches** record a file import or synchronization attempt.
- **Review items/decisions** capture duplicate, ambiguity, mutation and discard
  workflows.
- **Goals/reserves** are allocations and do not themselves imply cash movement.

Source records should be treated as immutable facts rather than workflow-state
containers. Linked, discarded, superseded and needs-review state should be
represented by the relevant links, decisions, supersession/tombstone records
and review items rather than one mutable source status. Manual entries should
also have lightweight provenance records. Opening balances should be explicit
adjustment transactions rather than unexplained mutable account fields.

The old first implementation slice was accounts, manual transactions, ledger
entries and monthly cashflow before imports. This is useful sequencing context,
not a mandated rewrite plan. Account and ledger schemas should retain a
workspace boundary from their initial design even in a personal-first product.

The documented ledger shape used positive amounts plus an explicit account
effect (`increase`/`decrease`), separate from business meaning. This handles,
for example, a credit-card purchase that is an expense while increasing a
liability balance. Transaction kinds previously included expense, income,
transfer, adjustment, allocation, refund and fee.

## CLI and external access

The existing docs treated the desktop HTTP API as an internal boundary and
deferred public API/MCP. The new owner direction supersedes that integration
priority: MCP is deprecated for now and a CLI is planned for human and agent
use.

The docs do not yet settle:

- Whether CLI commands invoke the same application interface as the web app,
  and whether they use an in-process library, local service or HTTP interface.
- Whether the CLI runs only against a self-hosted instance or also supports a
  local embedded mode.
- How CLI credentials, permissions, confirmation for writes, audit records,
  secret handling and revocation work.
- Stable command naming, machine-readable output, error conventions,
  pagination, idempotency and compatibility/versioning.
- Which write operations are supported initially and how agent-initiated data
  is reviewed or reversed.

The prior API guidance remains relevant as security context: external tools
must not receive bank credentials, provider tokens or raw integration secrets;
access should be scoped and auditable. The earlier read-only-first suggestion
is not automatically binding because the requested CLI explicitly needs to
add data. Define safe write semantics as part of CLI design.

## Explicitly deferred or undecided

The existing plan deferred real Open Finance integration, Fintual/other public
investment APIs, Splitwise, cloud sync, public API/MCP, annual tax preparation,
plugin systems, SaaS hosting and billing. The rewrite explicitly keeps MCP
deferred, and explicitly makes self-hosting the first deployment model with
SaaS as a possible later evolution. The other items remain candidates, not
commitments.

Important unresolved design topics from the prior planning docs:

- Detailed repository contracts, transaction consistency, importer/provider
  adapters, host adapters and the later CLI boundary.
- Source snapshot storage and encryption, local profile/unlock behavior, key
  recovery and any future sync encryption.
- Import adapters/pipeline, retries, tombstones and provider identity changes.
- Currency library and ISO 4217 handling; timezone/source-specific rules;
  budget rollover implementation; report calculation, caching and exports.
- Whether scenarios are derived views, persisted entities or both, and how
  assumptions are compared to baseline.
- Background jobs, retry/dead-letter policy, and eventual operations,
  deployment, logging, metrics, tracing and support tooling.
- SaaS authentication, workspace membership, account recovery and billing only
  when hosted deployment becomes real.

## Previous PoC material

Files under the former `docs/` tree describe the previous PoC and are obsolete
for current rewrite work. Do not treat them as product or implementation
requirements unless the user explicitly asks. The current rewrite context,
discovery, product decisions, acceptance criteria, backlog, domain model, and
planning notes are maintained under `desc/`.

## Next documentation work

Before implementing the first non-CLI slice, turn this handoff into focused
accepted decisions for:

1. Deployment and runtime shape for a self-hosted Rust service and web client,
   including the SaaS-compatible boundary.
2. The high-level Rust module structure, SQLx direction and exact-money
   envelope are accepted. Define detailed repository contracts, transaction
   consistency and PostgreSQL connection configuration. Assess SQLite only if
   a local desktop mode is pursued.
3. Initial product slice and data model — accounts, manual transactions, and a
   current-month budget are the accepted product scope in
   `desc/08-first-slice-plan.md`; finalize technical boundaries and numeric
   behavior before implementation. Reports, reconciliation, imports, and
   advanced rollover are later scope for this slice; the accepted basic close
   transition is included.

Before implementing the later CLI slice, define its interaction model, command
surface, output/error contract, trust boundary, and safe auditable write
behavior for humans and agents. These CLI decisions do not gate the first
web-and-ledger slice.

No application code is introduced by this context document.
