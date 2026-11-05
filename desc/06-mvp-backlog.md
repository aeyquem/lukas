# MVP backlog: product sequence

Status: Working planning draft. This orders the first-release behavior in
`03-spec-v1.md`, `04-mvp-walkthrough.md` and `05-mvp-acceptance.md` into
user-facing slices. It describes product scope and dependencies, not technical
implementation. Acceptance IDs refer to `05-mvp-acceptance.md`.

## Current decision: budget funding and expected spending pools

Confirmed allocation and shared-savings behavior:

- Greedy and paced strategies operate in the budget. Greedy funds goals in
  priority order, passing remaining allocatable money to the next goal. Paced
  assigns a monthly amount per goal, such as 1,000 to Vacations and 500 to
  Computer. The existing choice of automatic allocation versus confirmed
  suggestions remains per goal.
- Specifically funded goals receive direct budget assignments.
- The budget can instead assign money to a shared category, such as General
  Savings. Goal-to-buy goals check that category's balance without reserving
  individual amounts. Each can appear affordable on its own; warn when their
  combined costs exceed the shared balance.
- Buying one of these goals consumes the shared category balance once and
  records usage against that goal. The purchase is one expense, not separate
  category and goal expenses.
- Pools continue to identify expected payment sources, spending-order fallback
  and scenario restrictions; actual transactions identify the accounts used.

Goal funding comes from budget assignments. Eligible existing money enters
ready-to-assign once when tracking starts, subject to the agreed eligibility,
liability and count-once rules. The user can immediately assign it to a goal
or spending category. Later income is also counted once; carried assignments
retain their jobs without creating new funding.

A goal's pool identifies its expected source of payment, with spending-order
fallback in projections. Pool holdings and budget-assigned goal funding are
shown separately and are not added together. Assigning money to a goal does
not move money between accounts or change actual pool balances.

A specifically funded goal behaves as a budget category with a target
attached. Categorize its purchase to that goal category regardless of the
payment account. Ordinary budget assignment and expense recording remain two
visible steps; the product can offer them together in one purchase flow.
Selecting the goal category and recording goal usage describe the same expense,
not two expenses or deductions.

Example: with 5,000 already funded and no prior usage, assign another 1,000 to
Vacations, then record a 1,000 ticket expense in its category. Funded becomes
6,000, used becomes 1,000 and available remains 5,000. Without the additional
assignment, funded stays 5,000, used becomes 1,000 and available becomes 4,000.
The payment account/pool does not automatically add funding.

When linking a purchase to a specifically funded goal, choose whether to use
existing goal funding or assign additional budget money and spend it
immediately. Existing funding increases used and reduces available. Additional
funding increases funded and used together, leaving prior available funding
intact. Use available budget money or an explicit reassignment; preserve the
existing rule that over-assignment can be saved with a shortfall warning.
Account/pool choice alone does not determine the funding choice. A transfer
between accounts is not an expense or a second funding contribution.

Example: 5,000 already funded, plus 500 newly budget-assigned and transferred
to savings, plus 1,000 newly budget-assigned and spent on tickets, gives 6,500
funded, 1,000 used and 5,500 available. The transfer records location; the
budget assignment records its purpose, counted once.

This replaces separate pool funding and budget funding, and automatic goal
funding based solely on a purchase's pool. Earlier pool-reservation descriptions
are historical. Shared goals show cost, affordability and used amount;
specifically funded goals show assigned funding and funded progress. The
shared balance is not separate funding for every candidate goal.

## Planning principle

Deliver a useful monthly spending view early, then add budgeting and planning
that rely on trustworthy account activity. A slice is complete when the user
can reach its stated outcome with traceable financial facts and decisions.
Several first-release capabilities are broad; each can be delivered in smaller
increments while retaining the essential baseline described below.

## Ordered backlog

### 1. Establish accounts and starting balances — essential foundation

**User value:** Start tracking now without importing years of history, and
know what each account balance means.

**Baseline:** Create bank, cash, card, savings and investment accounts; enter a
dated opening balance (today by default); show account balances. Support
confirmed activity as the basis for calculated balances. Preserve opening
balance provenance. Reconcile any account type by showing the difference,
letting the user review and match activity, add an explicit adjustment for an
unexplained difference, or leave the process unfinished. Make adjustments
distinct, labeled account entries that correct the balance, show their effect
on available-to-assign, and are not ordinary category spending. Preserve
adjustment source and history. (AC-1.1–1.3)

Account creation requires a nonblank name, account type, base currency,
opening balance and date, and budget eligibility. Offer CLP, today and zero as
defaults, but ask the user to explicitly confirm the opening amount and its
currency; distinguish card debt from money held. A bank or card account may
hold several separately denominated balances/wallets; retain dated opening
amounts for the currencies actually tracked, without requiring unused wallets.
The base currency does not force conversion of those balances. Bank, cash,
savings and credit-card accounts default to budgetable; investments default to tracking only. Eligibility is editable per
account and controls budgeting only, without limiting categorization or
reporting. Institution, account reference, notes and pool assignment are
optional. Opening money enters ready-to-assign once under the existing
eligibility and debt rules. (AC-1.5)

**Dependency:** None. This establishes the account and date context used by
every later slice.

**Increment:** Add basic investment value and period variation without
holdings or investment activity. (AC-1.4)

### 2. Record activity and explain the current month's spending — essential first useful outcome

**User value:** Understand where money went, by category and account, and
inspect the movements behind a total.

**Baseline:** Enter transactions manually; use editable built-in and custom
categories; allow uncategorized activity; split expenses across categories;
add tags separately from categories and goals. Require account, date, amount
and activity type for manual transactions; transfers also require the other
account. Keep payee, category and note optional. Show confirmed activity for the
current calendar month by category and account. Treat card purchases as
spending on purchase date and bill payments as transfers. Handle refunds,
exposed fees and pending-to-posted card amounts as specified. Configure a
statement closing day and payment due day per card; show due-date reminders
without initiating payments. Clear reminders when recorded payments cover the
statement amount due; use the statement amount when available (with user
correction), or calculate it from recorded account activity through the
closing day. Also allow marking a reminder as read without changing payment
status. (AC-2.3–2.6, AC-3.1–3.3, AC-3.7)

**Dependency:** Accounts and opening balances. This is the first user-facing
vertical outcome and the basis for budget and projection history.

**Increment:** Add report-by-report choice of actual versus assigned period,
card close-date/payment-date rules and manual period assignment. Retain real
transaction dates. Add non-CLP reporting with statement CLP amount or a dated
estimate and visible source/rate; support cross-currency transfers. Preserve
bank/card accounts' base currencies and separate currency balances/wallets,
original purchases and actual bank conversion chains with dates/sources and
available or derivable rates. Conversion replaces the affected denomination
once, without a new expense or duplicate debt. Report estimates remain
distinct from actual conversions; current rates cannot change fixed converted
debt. Keep eligibility and pool grouping at account level. (AC-3.4–3.6,
AC-3.9–3.10)

Prefer bank-supplied purchase CLP amounts. For aggregate-only conversions,
keep bank totals authoritative and offer separately labeled derived
allocations: proportional suggestions across known eligible outstanding
converted portions, reviewed/confirmed and manually editable. Preserve settled
portions on partial conversions and the meaning of payments, opening debt,
fees and refunds. Reconcile allocation totals with visible rounding differences.
Update purchase/goal usage valuation once, without adding funding. Incomplete
composition leaves purchase valuation provisional without blocking the actual
aggregate conversion; require review rather than inventing components.

Foreign costs can have locked/floating CLP estimates from a selected rate and
optional spread, or an available card-network rate without an integration
commitment. Before purchase these affect forecasts/funding gaps. After
purchase, variable cost estimates change only the estimated funding gap;
recorded expense, funding and remaining amounts stay unchanged until confirmed
actual conversion/reviewed allocation updates the purchase once. Net worth
independently values real debt currently. Significant-change alert thresholds
remain to define. (AC-3.11)

Use currency-appropriate entry/display precision (whole CLP pesos, USD cents),
while preserving higher supplied precision/rates for inspection. Round derived
portions after calculation and distribute residual units visibly by largest
fractional remainder, with review order breaking ties. Valid manual changes
must retain the authoritative total; preserve bank-supplied purchase amounts,
refund signs and confirmed allocations across later view sorting. For example,
CLP 1,000 equal thirds become 334 + 333 + 333, with no extra expense. (AC-3.12)

**Reporting baseline:** The MVP includes a current net-worth snapshot: account assets less liabilities
across tracked accounts, including accounts excluded from budgeting. Use actual
dated balances, independently of assigned reporting periods. Show each balance's
date, original currencies, and a current CLP estimate with conversion source,
rate and date; never imply that missing conversion data is zero. Pool holdings
and budget assignments are views of those assets and are not added again.
Historical combined net-worth trends require a later valuation rule.

Spending views support date range, account, category and optional tag filters,
with underlying transactions inspectable. Category filtering of a split expense
includes matching portions once, excludes unmatched portions from totals, and
retains the parent relationship for inspection. Period comparisons show both
exact date ranges, absolute change and percentage change; when the prior value
is zero, percentage change is unavailable. A fully customizable report builder
and all exports are deferred beyond the MVP. (AC-3.8)

### 3. Import statements with review — essential input path

**User value:** Add existing bank and card activity efficiently without
letting unreviewed rows change balances or reports.

**Baseline:** Import one-account CSV and Excel files; review rows individually
or in bulk; keep unconfirmed rows out of balances and reports. Hold likely
duplicates for an individual keep, link/merge or discard decision. Preserve
the source and decision history. Flag rows missing required values or with an
unclear type; allow correction of any transaction field before confirmation,
preserving original imported values and the file-selected account association.
Let users directly edit confirmed
transactions while keeping prior details and changes inspectable and
preserving actual dates. For changes affecting a closed budget, update account
and transaction-report history, preserve the closed budget snapshot, and apply
the budget impact as a current-period adjustment where possible.
(AC-2.1, AC-2.2, AC-2.7, AC-2.9)

**Dependency:** Account setup, transaction meaning and the confirmed-versus-
unconfirmed boundary from slices 1–2.

**Baseline refinement:** Before review, select the account and, for Excel, the worksheet containing the
transaction table. Preview raw columns beside interpreted values. Let the user
correct column mapping, CSV delimiter, file-wide date format, decimal/thousands separators,
amount signs, or separate inflow/outflow columns. A valid interpreted date,
amount and transaction meaning are required; the source need not contain a
literal type column. An ambiguous date such as 03/04/2026 requires a selected
file date format rather than a different guess per row. Invalid amounts,
conflicting debit/credit values and unresolved direction remain flagged.

Unreadable files or files without a recognizable transaction table produce an
actionable file-level error and confirm no rows. In mixed files, selected valid
rows can be confirmed while unresolved rows remain in review with their original
values and explanations. Only a transaction classified as a transfer between
owned accounts requires the other account; an unknown counterpart stays in
review until resolved, without inventing an account or silently changing its
meaning. Missing category, payee or note does not block confirmation. (AC-2.10)

### 4. Group accounts into pools — essential context for goals

**User value:** See which money is associated with a shared purpose without
mistaking a grouping for a transfer.

**Baseline:** Create pools and assign each money-holding account in full to
one pool. Show original-currency breakdown and current CLP estimate with rate
and date. Permit card transactions to be associated with a pool while
excluding card liability and available credit from pool money. Show total,
calculated resource availability and a
user-set spendable amount; show the combined target amount for all linked goals
as an informational figure that does not reserve money or change availability.
Allow the user to select per pool whether the spendable amount is display-only
or used for planning. In projections, known dated obligations reduce simulated
pool resources on their due dates, not upfront, and each projected outflow is
counted once. (AC-4.1–4.4, AC-6.5)

**Dependency:** Account balances and currency context. Pool coverage depends
on those balances; shared-goal affordability depends on its budget category.

**Increment:** Partial account allocations and advanced allocation rules are
later scope.

### 5. Assign available money a job — essential zero-based budget

**User value:** Plan the calendar month so available money is assigned to
spending categories and goal destinations, while seeing what is still
unassigned.

**Baseline:** MVP assignments, goal targets and contribution plans use CLP;
foreign budget/goal-target currencies are later scope. Foreign cost components
can be estimated in CLP without changing that planning currency. (AC-5.9)
Start with eligible money not already assigned at period start
plus each income receipt once. Let the user set budget eligibility per
account. Create basic goal destinations with a name, target amount and
optional date so they can receive assignments. Assign to categories and goals,
never pools. Let users select per category or goal whether unspent assignment
carries forward or returns to unassigned. Carried assignment remains assigned
and is excluded from new unassigned money. Allow saving incomplete budgets
with the amount waiting for a job visible; allow saving over-assignment and
show the shortfall as `over-assigned by [amount]`. Support direct goal
assignments and assignments to shared savings categories; prevent the same
amount being counted twice. A card purchase uses its category budget when
made and virtually reserves only genuinely cash-covered budgeted money for
the later bill; covered money needs no second assignment at payment (AC-5.14).
Let each budget use actual transaction dates or assigned reporting periods as defaults, resolving each
purchase to exactly one budget month when mixed monthly settings conflict.
Category impact and bill reservation follow the same resolved month; preserve
real dates and independent report settings. Already-received income can fund
a later monthly budget once, without leaving that money available for another
assignment. Show card payments in their actual payment month's cashflow as
transfers, without moving the original purchase's budget impact or counting a
second expense. (AC-5.1–5.6, AC-5.8, AC-4.9)

Keep each budget period open until the user explicitly closes it; allow adding
and editing transactions dated in an open period after its calendar month ends.
Require carryover review at close. Allow a more deliberate correction after
close: edit the transaction directly with its actual date and audit history
intact, preserve the closed budget snapshot, and apply the budget impact in the
current period where possible. (AC-5.7)

Automatically revalue eligible foreign assets for CLP budget funding in the
MVP. Explain FX differences as valuation adjustments: gains add ready-to-assign
once, losses expose shortfall/over-assignment without changing fixed jobs or
funded goals. Show current valuations on tracking-only accounts without budget
impact. Retain actual wallet denominations, bank-known incoming conversions,
source/final-replacement rules and once-only receipts/transfers; do not apply
this rule to warning-only foreign expense estimates. Preserve closed snapshots
and apply current effects once. Automatic-versus-reviewed user choice is later
scope. (AC-5.10)

Explain owned transfers across the budget boundary as funding effects:
tracking-only to budgetable adds funding once; the reverse removes it once.
Transfers within either group have no budget-boundary funding effect;
independent FX differences and exposed fees still follow existing rules once.
Financial reports keep
the movement classified as a transfer, without new income/spending, transfer
categories or automatic goal usage. Leave assignments and funded goals fixed,
show resulting over-assignment, and retain real dates and closed-period
correction behavior. Initial signed balances follow AC-5.13; subsequent
positive-credit consumption follows AC-5.18; returns retain transfer treatment. Purchase coverage
follows AC-5.14. Payments of already-recognized debt keep their covered or
paid-uncovered treatment under AC-5.19 without another ordinary boundary
deduction for either portion. Unrelated asset transfers keep the boundary rule. (AC-5.11)

Preview funding effects before confirming account eligibility changes. Apply
the current eligible positive asset balance prospectively once: add it when enabling
budgetability, remove it when disabling. Keep assignments/funded goals fixed
and show over-assignment where needed. Do not create money movement,
income/spending, category/goal usage or changed bank data/report history;
preserve real dates and closed snapshots. Retain existing FX asset rules
without replaying original receipts or prior adjustments. Recognized debt and
coverage survive eligibility changes under AC-5.19. (AC-5.12)

Initial funding includes actual signed budgetable balances: negative bank,
cash or savings balances reduce it, actual positive card credit adds it,
and unused credit/overdraft limits add nothing. Preserve the opening card-debt
rule and count subsequent transactions, category impacts and bill reservations
once; do not deduct debt twice or automatically call card credit income.
Credit consumption follows AC-5.18; debt-bearing eligibility changes follow
AC-5.19. Purchase coverage follows AC-5.14. (AC-5.13)

Record a valid actual card purchase in full even when its category has too
little available money. Spending reflects the whole purchase; actual positive
card credit is consumed before new debt under AC-5.18. Category availability
may become negative. Reserve only the genuinely cash-covered portion still
owed for the bill and show uncovered borrowing as debt. Existing credit spent
without category coverage is unfunded asset use, not additional card debt. Do not automatically take ready-to-assign money or money
from other categories. An explicit later assignment or reassignment backed by
eligible money can cover the shortfall and increase the reservation once,
without another expense or debt increase. An unsupported assignment may still
be saved with an over-assignment warning, but cannot create cash-covered bill
reservation. Budget/category/coverage shortfalls are warnings, never hard
limits on recording actual purchases or payments the bank allowed. Lukas
records activity; it neither initiates nor authorizes banking transactions.
Required fields and existing import confirmation, ambiguity and duplicate
review rules still apply. Preserve opening-debt, once-only, FX valuation and
fixed-assignment/funded-goal rules. Partial-payment coverage follows AC-5.15;
refund coverage follows AC-5.16 and positive-credit consumption AC-5.18;
ordinary goal-linked refunds follow AC-5.20; exceptional/excess cases remain
open. (AC-5.14)

A bill reservation commits the same budgeted cash, usually held in a bank
account, to the later card payment. It is not money held on the card, a separate
account or a second budget assignment. For a same-currency payment between
budgetable accounts, use the card's existing payment coverage first and leave
unused coverage committed. Payment consumes coverage; it does not make that
cash ready to assign. Opening debt already deducted from initial funding has
existing protection and must not be funded again.

Record any amount paid beyond existing coverage without blocking or silently
sweeping other categories. For the portion repaying actual debt, distinguish
the paid-without-prior-coverage portion
from debt still outstanding: these are parts of the original funding gap, not
new shortfalls added on top. Later explicit funding covers the already-paid
portion without reserving it again; only the still-outstanding portion can
receive a bill reservation. Spending and category overspending are unchanged
by the payment itself. These are Lukas budget-coverage effects, not claims
about the bank's allocation among purchases. Actual statement evidence governs
payment reminders; preserve real payment dates and closed snapshots. Refund
coverage follows AC-5.16 and positive-credit consumption AC-5.18; ordinary
goal-linked refunds follow AC-5.20, with exceptional/excess cases still open.
(AC-5.15)

For a refund linked by source evidence or the user to an underfunded card
purchase, first repair that purchase's uncovered amount. Release corresponding
unpaid bill coverage only after that shortfall is cleared. Reduce spending
in the original purchase's category once and restore that category's
availability; create no income or extra ready-to-assign money. An unlinked
refund instead keeps the existing user-selected category behavior (AC-2.5);
do not apply a linked purchase's coverage restoration to another category.
Record the actual card balance change and refund date independently of this
budget-coverage ordering.

For a fully funded purchase refunded before payment, reduce its matching
unpaid reservation and restore category availability. After full payment, an
actual refund can create positive card credit and restore category availability,
but do not release the consumed reservation again. Reservation changes never
move money. Do not infer the bank's application to unrelated purchases; retain
source/user links, independent report periods and existing closed-budget
correction rules. Ordinary goal-linked refunds follow AC-5.20;
exceptional/excess goal-refund cases remain open. Positive-credit consumption
follows AC-5.18, and returning owned money retains transfer
treatment. (AC-5.16)

Outstanding foreign card debt stays in its actual currency until paid or
actually converted. Current FX updates its CLP net-worth valuation and the
estimated coverage gap, not the recorded purchase valuation, fixed budget
assignments, genuinely funded bill reservation or ready-to-assign. A locked
purchase estimate cannot freeze the actual liability. Show the coverage warning
alongside ready-to-assign: that figure alone does not guarantee current foreign
debt coverage. Automatic eligible foreign-asset revaluation remains separate.

Combine existing uncovered debt and the incremental FX coverage difference
into one explained estimated shortfall. Do not count that difference again as
a funding deduction, new expense or another full shortfall. Apply revaluation
only to still-outstanding foreign portions; fixed converted CLP debt and
settled portions are not revalued again. Favorable FX does not automatically
release committed cash; changes to coverage require actual conversion evidence
or an explicit funding action. Budget/coverage warnings do not block actual
activity or silently sweep another category.

An actual bank conversion or confirmed aggregate allocation replaces the
provisional coverage difference with the actual difference once. Apply the
existing source, category/goal usage, genuine coverage and closed-period
correction rules without another purchase or automatic funding. For opening
debt, adjust its existing protection/funding once rather than creating new
spending or requiring the already-protected amount to be funded twice.
(AC-5.17)

For spending from a budgetable card balance with real positive credit, use
that credit first in the affected currency balance. Record the full purchase
as spending and category use once, with its existing goal link if present.
Only the newly borrowed remainder can require bill reservation. Funded
category money first covers the purchase portion paid from existing credit;
only remaining genuinely funded purchase coverage can reserve new borrowing. Existing credit is an actual asset, not unused
borrowing capacity, and its consumption creates no income or additional
ready-to-assign. Residual positive credit remains an account asset.

Do not count credit again if it already contributed opening funding or arose
from a refund's category restoration. Category assignments still determine the
purchase's funding; the physical credit need not have been separately assigned
to that category. An underfunded category can become negative: distinguish
existing credit used without budget coverage from uncovered new debt, without
inventing reservation or duplicating the gap. Warnings never block valid actual
activity. Credit in another currency is not an offset unless actual bank
conversion/application evidence supports it.

A payment exceeding total debt can leave real positive card credit. Its
non-debt portion is an owned transfer into that asset, not spending, income
or another uncovered debt payment. Existing source, actual-date, eligibility,
FX, refund and closed-budget rules remain in force; do not infer the bank's
allocation among unrelated purchases. (AC-5.18)

For a same-currency example, bank CLP 700 plus actual card credit CLP 300
provide CLP 1,000 funding. Assign CLP 500 to Groceries, leaving CLP 500 ready
to assign. A CLP 500 card purchase uses the credit CLP 300 and creates only
CLP 200 debt: bank stays CLP 700, Groceries availability becomes zero,
reservation is CLP 200 and ready-to-assign stays CLP 500. Spending is CLP 500
once; no extra funding or assignment is created from the consumed credit.

Preserve already-recognized liabilities and their coverage when a card or
negative-bank account becomes tracking-only. Positive assets leave funding
prospectively, but debt protection, reservations and uncovered amounts remain
until settled under existing rules. Enabling participation recognizes only
previously unrepresented debt once; preserve prior coverage and do not create
funded reserves without backing. A later payment of already-recognized debt
keeps its once-only treatment: covered portions consume protection, and
uncovered portions retain AC-5.15's paid-uncovered gap treatment. Neither
portion receives another ordinary boundary-outflow funding deduction.
Unrelated asset transfers and positive-credit portions keep existing rules. Retain preview/confirmation, warnings,
source/actual-date/closed-snapshot rules and existing FX/payment/refund/credit
behavior. (AC-5.19)

**Dependency:** Confirmed activity and income semantics, categories and the
distinction between account balances, pool grouping and assignments. This
slice establishes the budget assignment capability used by goal tracking in
slice 6.

**Increment:** Detailed period correction flows may be refined during product
design.

### 6. Track funded goals and shared-savings purchases — essential planning capability

**User value:** Fund specific goals or compare possible purchases against a
shared savings category, while understanding expected payment sources.

**Baseline:** Specifically funded goals behave as budget categories with
targets. Categorize their purchases to those categories regardless of payment
account. Offer assignment and expense recording together when helpful while
keeping both steps visible and counting the expense once. Greedy allocation
funds goals in priority order; paced allocation assigns a monthly amount per
goal. Retain the per-goal automatic-versus-confirmed-suggestion choice.
Goal-to-buy candidates use a shared budget category, such as General Savings,
without individual reservations. Show individual affordability and
combined-cost conflicts against its balance. A purchase consumes that category
balance once and records goal usage without a second expense. Show targets,
optional dates, monthly saving needs and specifically funded goals' saved,
used and available values with saved/target progress. Pools identify expected
payment sources and are not an additional funding source. Keep manual goal
links and explicit exceptional saved reductions reviewable. (AC-4.5–4.12;
AC-4.8 covers existing versus additional purchase funding.)

A refund of ordinary goal-linked purchase usage reverses that linked used
amount once and restores the original budget category's availability once
under AC-5.16. It does not add saved/funded money, change the target or advance
funded progress. For a specifically funded goal, used falls and available rises
while net saved/funded stays fixed. For a goal-to-buy, usage falls and
the shared category is restored once; no individual goal funding is created.

Keep the original purchase, refund and confirmed goal/category relationship
inspectable. Use the accepted CLP valuation/source hierarchy without mixing
original foreign amounts into CLP goal totals. Actual refund dates and existing
closed-budget correction rules remain in force. Card debt, reservation and
positive-credit effects follow AC-5.16; whether the bill was paid does not
change this ordinary goal-usage reversal. Never restore the category twice or
add the refund to ready-to-assign as another income/contribution.

A refund of an exceptional saved-reduction transaction must refer to and
reverse its original exceptional effect, not pretend it reversed ordinary usage
or provided a new contribution; the exact exceptional restoration rule remains
open. Refunds or FX differences exceeding remaining linked usage also remain
open. Correcting a purchase-portion goal link follows AC-5.22; explicit funding
reassignment follows AC-5.21.
(AC-5.20)

For a specifically funded goal with saved CLP 5,000, used CLP 1,000 and
available CLP 4,000, an ordinary linked CLP 300 refund leaves saved CLP 5,000,
used CLP 700 and available CLP 4,300. Funded/target progress is unchanged.
For a goal-to-buy with CLP 1,000 used and CLP 4,000 available in its shared
category, the same linked CLP 300 refund leaves usage CLP 700 and shared
availability CLP 4,300, without an individual saved amount or duplicate
category restoration.

An explicit funding reassignment away from a specifically funded goal reduces
its net saved/funded amount, leaves used unchanged and recalculates available
as funded minus used. Preserve the original contributions and explicit
reassignment in history. This is a change of budget purpose, not an account
movement, income, expense, ordinary usage or an exceptional saved-reduction
transaction. Funded/target progress reflects the reduced net funding.

A direct reassignment increases the receiving category/goal assignment once;
a specifically funded recipient's funded amount increases once. Do not also
deduct ready-to-assign for the same direct reassignment. If funds are instead
returned to ready-to-assign, release them once; a later assignment consumes
that ready-to-assign once. No new budget money is created. Reducing a shared
savings category changes its balance and goal-to-buy affordability once; no
individual saved amount exists to reduce for each candidate.

Allow reassignment beyond available funding with a clear shortfall warning,
even when the source goal's available amount becomes negative. It does not
undo historical usage or make the receiving assignment genuinely backed.
Show the source/recipient consequences as one funding gap, not two deficits
added together; do not invent cash or bill coverage. Fixed actual balances,
transaction dates and closed-budget snapshots retain their existing rules.
This is the warning policy for explicit funding reassignments only; corrected
links and exceptional saved-reduction transactions/refunds remain separate
questions. (AC-5.21)

For a goal with saved/funded CLP 5,000, used CLP 1,000 and available
CLP 4,000, explicitly moving CLP 500 to another budget category/goal leaves
funded CLP 4,500, used CLP 1,000 and available CLP 3,500. Moving CLP 4,500
instead from the same initial state leaves funded CLP 500, used CLP 1,000 and
available CLP -500. Warn about that single CLP 500 coverage shortfall; the
receiving CLP 4,500 assignment does not prove that all of it has backing.

Each purchase portion may have at most one explicit, manual goal link,
separate from tags. An unsplit purchase has one portion; splitting permits
one parent purchase to support multiple goals. Ordinary/shared-category
portions may remain unlinked. A specifically funded goal category mandates the
matching goal association and corresponding goal effect: its portion cannot
be unlinked or point to another goal. Selecting that goal category is the
manual association, not another expense or an automatic suggestion.
All portions reconcile to the parent amount, and only linked portions record
their respective goal usage. The parent amount is not added again to every
goal, account effect or budget spending total. Automatic goal-link suggestions
remain later scope.

A portion linked to a specifically funded goal uses that goal's budget
category. A goal-to-buy portion uses its shared source category and its own
goal link: multiple portions may use the same shared category but different
goals. The shared category falls by the purchase's applicable portions once,
without creating individually saved amounts for the candidates. Funding,
usage and available amounts follow each associated portion; explicit funding
assignments/reassignments count once under AC-4.8 and AC-5.21. Ordinary linked
refund reversals retain AC-5.20 for the affected usage. Correcting a portion's
goal link is an auditable correction: reverse that portion's usage from the old
goal and apply the same usage to the new goal. If the goals use different
categories, correct the portion's category with the link. Keep the expense,
account effect and portion amount unchanged; do not create a new assignment or
funding. Preserve the prior link and correction history. Retain the actual
transaction date and use the established current-period correction process for
closed budgets, leaving their snapshots intact. Allocation of refunds across
portions remains open; do not infer it from the parent amount. (AC-5.22)

A CLP 300,000 marketplace card purchase is split into a CLP 220,000 suitcase
portion categorized to Vacations and linked to that goal, and a CLP 80,000
camera portion categorized to Photography and linked to that goal. Record
one CLP 300,000 account effect and CLP 300,000 budget spending in total, with
Vacations usage CLP 220,000 and Photography usage CLP 80,000. Existing
positive-credit, debt and reservation rules determine the actual balance and
coverage effects; do not count the parent and portions as separate purchases.

For two goal-to-buy candidates using General Savings, corresponding portions
can both use General Savings while carrying different goal links. General
Savings falls by CLP 300,000 once, and each goal receives only its linked
portion's usage, without new individual saved/funded balances.

**Dependency:** Budget assignments from slice 5, account/pool context and
reviewed transaction activity. Shared-savings purchases require a funded
shared category.

**Display:** Shared goals show cost, affordability and used amount; specifically
funded goals show assigned funding and progress.

Support existing goal funding or additional budget funding spent immediately
when linking a purchase, without inferring funding from the payment pool.
Automatic transaction suggestions and partial pool allocations remain later
scope.

### 7. Forecast cashflow and recurring expenses — essential projection

**User value:** See how current patterns may affect future months and how
dated goals fit, with assumptions visible.

**Baseline:** Let the user choose historical lookback and forecast horizon.
Show estimated income, spending, net cashflow and progress toward dated goals.
Goal savings reduce money left after expenses but remain separate from
spending. For scheduled goal purchases, deduct each cost from the preferred
pool and carry projected balances forward. If it exceeds the pool's resources,
cover the shortfall in subsequent pools using the user's spending priority and
show each pool's contribution. Reserve targets are warning thresholds; fallback
can cross them but must show the shortfall. Stop projected drawdowns at zero
and show any remaining cost as unfunded. Apply each known dated obligation to
projected pool balances when it occurs, not upfront, and do not subtract it
again if it is already represented as a projected outflow. Support simple
recurring expense schedules (every N days, day of
month, month-end, ordinal weekday), an editable estimate per occurrence,
skip/reschedule and end date. Let the user link a confirmed/imported actual to
a forecast occurrence. When history and a recurring rule overlap, let the user
choose per projection which source takes precedence. Separately, when a
scheduled goal purchase may overlap a recurring or history-based estimate, let
the user mark whether they describe the same cost or different costs; count a
same-cost pair once and different costs twice, preserving both sources and the
decision without silent matching. Keep estimates distinct from actual history.
(AC-6.1–6.4, AC-6.6)

**Dependency:** Dated transactions, categories, goals, and budget/cashflow
definitions.

**Increment:** More detailed scenario comparison and multi-transaction events
(such as a trip with several expenses) are later scope.

## Release boundary

The essential baselines above together define the current first-release
candidate; the increments are refinements within that release where called
out, except items explicitly labeled later scope. Shared-expense tracking and
Splitwise read/write, CLI, Open Finance, investment activity/holdings,
automatic goal suggestions, partial account-to-pool allocations, user-defined
pool/account restrictions and unlock events, FX scenario/range views, and
multi-transaction recurring-event templates remain after the first release.
The already-agreed floating-cost funding-gap warning remains MVP scope.

## Decisions needed before detailed planning

Pool/goal funding is settled: budgets fund goals, while pools show expected
payment sources. The first release includes simple spending-priority fallback
for projected goal purchases, with reserve thresholds shown as warnings.
User-defined account/pool restrictions and unlocking events are deferred until
after the first release. They remain scenario assumptions and do not change
actual accounts or monthly budgets.

The settled pool/goal model and projection release placement remain as above.
CLP planning and locked/floating foreign-cost estimates are specified in
AC-5.9 and AC-3.11. Automatic current-rate revaluation of eligible foreign
assets for budget funding is MVP behavior (AC-5.10), distinct from warning-only
foreign expense estimates. User-selectable automatic versus reviewed asset
valuation is later scope. Foreign-debt valuation and budget coverage are
settled in AC-5.17. Significant-change alert thresholds and unusual source-total
precision remain open. Ordinary split rounding and residual distribution are settled in
AC-3.12.

FX range display is accepted alongside fixed goal funding/progress, as
later-release FX-R1 describes: scenario backing coverage and future cost
ranges remain informational, use consistent scenarios and do not earmark or
double-count wallet balances. The FX scenario/range view is later scope;
calculation source/method, historical lookback settings and trend treatment
remain open for that later capability. No provider integration is promised,
and the MVP funding-gap warning remains as agreed.

Account setup and import correction requirements now follow the refinements
in AC-1.5 and AC-2.10. Current net-worth reporting is included in the MVP; all
exports are deferred. Additional reports and goal/projection comparisons can
be refined after the specified first-release flows.

See `03-spec-v1.md` and `05-mvp-acceptance.md` for the full product rules and
observable behavior behind this sequence.
