# First implementation slice: accounts, transactions, and monthly budget

Status: Product scope and interaction decisions accepted by the owner.
First-slice technical and finance contracts are decided in
`desc/13-first-slice-decisions.md` and passed independent review. The plan is
ready for implementation; required runtime evidence remains in
`desc/12-pre-implementation-checklist.md`.

## User outcome

The owner can create accounts, record manual financial activity, and assign
available money to categories for the current month. Account balances reflect
dated activity, and transactions update budget activity. The first instance is
owner-only; the owner may manage personal or household finances without
separate member identities.

## End-to-end user path

1. The app opens on an empty dashboard. No onboarding flow is required for the
   first slice.
2. The owner creates an account in one form. Required information includes its
   name, type, base currency, and whether it participates in the budget. The
   form records dated opening balances. Bank, savings, and investment accounts
   track one currency each; card and cash accounts may track several
   currencies. Cash may also be split across separate accounts. The base currency
   does not convert wallet balances. Support bank, cash, savings, card, and
   investment accounts; investment accounts default to tracking-only. Account,
   tracked currency balances, and their opening entries are created atomically.
3. The owner records income, expenses, or transfers manually. Require account,
   actual date, amount with currency, and activity type; the currency must be a
   tracked balance on that account. Transfers also require the other account
   and the amount/currency received there. Same-currency transfers move equal
   amounts; cross-currency transfers retain both actual amounts and their
   inspectable conversion relationship. Category, payee, and note may be
   omitted. A foreign-currency expense without a confirmed CLP statement value
   remains visible in its original currency; do not invent a transaction-date
   estimate. Show its category's CLP activity and remaining as incomplete, with
   known CLP subtotals and unresolved original amounts. Keep assignments
   savable, and block close while required values are incomplete. A confirmed
   CLP value completes the same expense once; never silently count it as zero.
4. The owner assigns money to categories in the current-month budget and can
   see transaction activity and remaining amounts. The first calculation
   starts from zero; initial available-to-assign adds eligible signed opening
   balances once. For an eligible foreign-currency opening liability, use its
   separate signed CLP funding basis once, preserving the original currency
   balance and excluding daily debt FX deductions. If that basis is missing,
   funding and ready-to-assign are unavailable. Credit limits do not count.
   Card accounts use their actual balances and transactions.
   A new calendar month's budget accepts assignments even while the prior
   month remains open. Calculate the new month forward from the most recent
   explicit close and intervening activity, even if the prior month has not
   been closed. Closing that prior month later makes its computed values the
   checkpoint for subsequent calculations; recalculate the new month from that
   checkpoint without double-counting activity or changing assignments already
   made there.
5. The owner can edit account, transaction, and budget values. For this MVP,
   edits overwrite stored values; change history and rollback are deferred.
6. At explicit month close, carry existing unassigned ready-to-assign forward,
   preserve the closed snapshot, and do not reseed the same money from account
   balances. For each category, the user chooses whether its signed remainder
   returns to ready-to-assign or carries in that category for the next period
   (AC-5.5). Preserve the sign: a negative remainder returned to ready-to-
   assign reduces it; carried in the category, it remains a deficit there.
   Do not close automatically at calendar month-end. Block close while
   required values are incomplete. The first-slice close sequence and other
   transition details are resolved in `desc/13-first-slice-decisions.md`.
   Advanced multi-period rollover remains open.
   An open month remains calculable from the most recent explicit close and
   intervening activity. Until it closes, provisionally carry signed category
   remainders in the same category into the next month. At close, the user can
   keep each carry or release it to ready-to-assign, then later months
   recalculate from that close without double-counting activity or changing
   assignments already made.

## Interaction decisions

- Use native HTML and basic CSS. Visual polish and formal accessibility review
  are not first-slice goals; defer accessibility review until the final UI
  design. Keep validation errors clear.
- In a multi-field creation form, submitting saves the complete item after
  validation. In a grid cell, explicit save and cancel actions preserve or
  restore the field baseline.
- Account creation, its tracked currency balances, and their opening-balance
  entries are one atomic operation.
- When a transaction is dated before the opening entry for its account and
  currency, ask whether to apply its effect to the current balance or preserve
  the current balance by adjusting that currency's opening balance by the
  inverse of the transaction's effect. The owner explicitly chose this prompt
  behavior so activity already represented in a starting balance can be entered
  without silently changing today's balance.
- When a transaction has the same date as its wallet's opening balance and
  inclusion is unknown, require the same initially unset apply-effect or
  preserve-current-balance choice for each affected wallet. Do not infer event
  order from dates, IDs or record order.
- If a save fails, show an alert with details and Retry/Cancel choices. A timed
  out write can have an unknown outcome; Retry reuses its exact request and
  idempotency key. Cancel may dismiss the alert but cannot cancel a committed
  write or clear the unresolved command; retain it for a later exact retry.
  For a definite rejection, Cancel may discard the draft.
- Use ordinary pagination for transaction lists. Search and date-range queries
  can be added later.

## Budget scope

- Include a current-month budget with category assignments and transaction
  activity/remaining amounts.
- Seed an editable starter set of expense categories: Vivienda, Alimentación,
  Transporte, Servicios y comunicaciones, Salud, Educación, Hogar, Vestuario y
  cuidado personal, Restaurantes y ocio, Suscripciones, Seguros e impuestos,
  Mascotas, Regalos y donaciones, Ahorro general, and Otros. Owners can add
  categories; uncategorized expenses remain visible as uncategorized.
- Use signed actual balances from budget-participating accounts to determine
  initial available-to-assign. A negative balance reduces it; a positive
  balance adds to it. Unused credit and overdraft limits do not contribute.
- Convert eligible non-CLP asset balances for CLP budget funding using a daily
  mid-market exchange rate. Show the rate and its effective date with the
  valuation. On weekends and holidays, use the most recent published daily
  rate and show its effective date. Frankfurter v2 is the selected provider;
  during an outage use the latest stored rate and label it stale. If none is
  stored, show foreign valuation as unavailable; do not treat it as zero or
  return a partial CLP budget-funding total.
- Card accounts are accounts with actual balances. Card purchases affect
  budget activity through their transactions; paying a card is a transfer, not
  a second expense. Do not add card limits as available funds.
- The basic explicit close transition above is in scope. Advanced multi-period rollover,
  goals, pools, bill-coverage workflows and prospective account-eligibility
  changes are not part of this slice. Calendar rollover does not auto-close.
- The full set of budget domain behavior in `desc/05-mvp-acceptance.md` remains
  the product direction for later work. Implement only the current-month
  behavior above in this slice; do not infer that all later budget workflows
  are included.

## Database and deployment requirements

Use PostgreSQL for the first slice and initial self-hosted deployment. Local
development runs the frontend, Rust API, and PostgreSQL with Docker Compose.
The backend and frontend have separate Dockerfiles; production does not require
Compose. Use SQLx with explicit SQL and versioned migrations, applying pending
migrations at self-hosted startup with SQLx migration locking enabled. Use
`rust_decimal::Decimal` as the bounded exact calculation type and PostgreSQL
`NUMERIC` for persistence; reject out-of-envelope values and operations without
silent rounding or truncation. For derived CLP foreign-asset valuations,
multiply the exact balance by the exact rate and round once to a whole peso,
nearest with midpoint away from zero. SQLite remains a possible later local
desktop option, not a first-slice requirement.

## Initial unauthenticated access boundary

The first self-hosted instance is reachable from the same machine only. Bind
the frontend's published host port to loopback; do not publish the backend
port to the host or expose either service to the LAN. The browser uses a
same-origin `/api` route through the frontend container to the backend on the
private container network. The frontend accepts only its configured loopback
Host; disable CORS and require the exact configured Origin for browser writes.
Do not place this unauthenticated instance behind a public or LAN-facing proxy.
Any wider network access requires a separately reviewed access-control design
first.

## Explicit exclusions

- CSV/Excel import, duplicate resolution, and import-review workflows. The
  owner's reference to imported activity explains why opening balances need
  dates; it does not bring imports into this slice.
- Reports, reconciliation, CLI commands, and separate member identities.
- Advanced rollover/carryover, goals, pools, projections, investment value
  history, Open Finance, and bank/investment provider integrations. The
  selected Frankfurter v2 daily FX-rate lookup is included because eligible
  foreign-asset budget valuation requires it.
- Transaction-date CLP estimates for non-CLP activity and advanced conversion
  allocation. Preserve original amounts/currencies; use a supplied final CLP
  statement amount as an actual value when present. The reviewed proposal for
  a foreign expense before its final CLP value is known is described below; do
  not silently treat an estimate as recorded spending or assume zero spending.
- Historical versions or rollback for edited values.

## Implementation handoff

The detailed data model and persistence/repository contracts; money, date, API
and concurrency semantics; frontend/API/DB runtime wiring; server-side
workspace resolution; funding derivation; and correction/close behavior are
decided in `desc/13-first-slice-decisions.md`. Validate those contracts through
the real local Compose/PostgreSQL path using
`desc/12-pre-implementation-checklist.md` during implementation.

### Accepted incomplete-value behavior and resolved finance/date decisions

The incomplete-value behavior below is accepted. The remaining first-slice
finance/date contracts are decided in `desc/13-first-slice-decisions.md`. An
unvalued foreign expense
posts its actual original-currency ledger effect. Its category's CLP activity
and remaining are incomplete: show known CLP activity as a partial subtotal
and list unresolved original amounts. Dependent aggregate availability is
incomplete. A confirmed CLP value completes the existing expense once, without
another expense or account debit. Ready-to-assign may be complete only when its
own funding inputs are complete; show a separate warning while an expense
value is pending. Assignments remain savable intent even when their coverage is
unknown. Block close while required values are incomplete.

When an eligible foreign opening liability has no confirmed CLP opening funding
basis, funding and ready-to-assign values are unavailable, with reason
`opening_liability_clp_basis_missing`; do not return a partial total or deduct
a current FX estimate. Preserve the signed original balance and require a
separately supplied signed CLP opening funding basis, applied once. This rule
is accepted. A direct correction to the foreign opening-liability amount must
include the user's supplied or explicitly confirmed signed CLP funding basis
in the same update; never derive the revised basis from daily debt FX. Store
the basis and provenance separately, and update it atomically with a
principal-changing apply/offset correction as specified in
`desc/13-first-slice-decisions.md`. Asset-rate unavailability is a separate
reason.

For an entry before the opening date, the accepted per-wallet apply-effect or
preserve-current-balance choice remains mandatory. For an entry on the same
date where inclusion is unknown, require the same explicit, initially unset
choice for each affected wallet. Do not infer a time or order. Same-day
treatment and its per-wallet persisted representation are specified in
`desc/13-first-slice-decisions.md`.

The accepted close preserves the closed snapshot and does not occur
automatically. Existing unassigned ready-to-assign carries forward. The user
chooses per category whether its signed remainder carries in that category or
returns to ready-to-assign (AC-5.5). Month initialization, chronological close
eligibility, atomic snapshot/checkpoint, and incomplete-value handling are
specified in `desc/13-first-slice-decisions.md`. Advanced multi-period rollover
remains outside the first slice.

**Selected FX provider: Frankfurter v2.** Its documented public API provides
daily mid-market rates, includes CLP coverage, needs no API key, permits
commercial use of its blended rates, and encourages local storage of daily
observations. Check at backend startup and once daily while it remains running;
fetch/store only when a newer published rate is available. On weekends and
holidays, use the latest published rate and show its effective date. If the
provider is unreachable, use the latest stored observation and mark it stale.
When no observation is stored, mark foreign valuation unavailable without
returning a partial CLP budget-funding total. Frankfurter can also be
self-hosted. Its blended rate may change slightly as new provider data arrives,
so preserve the exact rate observation used for each valuation.

Google's documented `GOOGLEFINANCE` option is a Sheets function; Google says
historical data cannot be retrieved through the Sheets API or Apps Script. XE
offers a documented daily mid-market API but its daily plan is paid and
requires account credentials. Scraping would add page-format and access-policy
fragility when a documented API is available. References:
[Frankfurter API](https://frankfurter.dev/),
[Frankfurter currency coverage](https://frankfurter.dev/currencies/),
[Google Sheets GOOGLEFINANCE help](https://support.google.com/docs/answer/3093281/googlefinance),
[XE Currency Data API plans](https://developers.xe.com/docs/currency-data-api/data-plans).
