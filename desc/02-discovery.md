# Product discovery: initial scope

Status: Working draft. This records discovery input and hypotheses; it is not
an implementation or architecture decision.

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
- Pools identify expected payment sources. Simple spending-priority fallback
  is included in first-release projections; user-defined scenario restrictions
  and unlock events are later scope. Actual transactions identify the accounts
  used. Known dated obligations reduce projected pool balances when they
  occur, not upfront; count each projected outflow once. Goal targets remain
  informational and are not obligations.

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

## Current decision: one budget month per purchase

Every purchase affects exactly one budget month while retaining its real
transaction date. Each budget's chosen date basis supplies defaults; conflicting
monthly settings require resolution and must not count a purchase twice or
omit it. Reports choose their period basis independently. Category use and a
card purchase's bill reservation follow the same resolved budget month.

Income already received can fund a later monthly budget. Count the receipt
and its assignment once; money assigned for November is not simultaneously
available for another assignment in October. An actual-date cashflow view
shows a November card payment in November as a payment/transfer, with its bank
outflow and debt reduction. The purchase remains spending once in its assigned
budget month; paying the bill does not create a second expense or move that
purchase to the payment month. Preserve closed snapshots and the agreed
current-period correction process when attribution changes affect a close.

## Current decision: account currencies and actual conversions

A physical bank or card account has a base currency and may hold multiple
separately denominated balances/wallets. The base currency is distinct from
CLP reporting currency. Budget eligibility and pool grouping remain choices
for the account, not separate wallet settings. Current account/pool/net-worth
CLP totals include each relevant balance once and never add a base-currency
summary again as another balance; card debt remains excluded from pool money.

Retain the original purchase amount/currency and each actual bank conversion's
input/output amounts, currencies, rate when available or derivable, date and
source. Banks may convert JPY/EUR purchases immediately to CLP, retain the
spent currency without conversion, or convert EUR to USD debt and later USD
to CLP at the statement cycle. An actual conversion changes the denomination
of money/debt on its conversion date; it does not create another expense or
leave both converted and replaced debt outstanding. The purchase date remains
unchanged. Reporting estimates alone do not constitute actual conversions.

Spending still uses a final attributable CLP statement amount when available,
otherwise a transaction-date estimate with source/rate. Final evidence replaces
its prior estimate or difference once, with the existing review and closed-
period correction rules. Preserve earlier observations. Current rates may
revalue an outstanding foreign-currency balance for display, but do not change
actual fixed converted debt. Do not invent fees or conversion steps.

## Current decision: allocating aggregate currency conversions

When the bank supplies a purchase-level CLP conversion, that exact amount
takes precedence over a Lukas allocation. If the bank supplies only an
aggregate conversion, its input/output amounts are authoritative for the
account. Internal purchase/category/goal allocations are separately labeled
as derived from the statement conversion, never as bank-supplied purchase
amounts. Use the aggregate's effective ratio only for eligible portions
actually included in that conversion; allocations reconcile to the bank total
and any rounding discrepancy is visible.

With known converted-balance composition, suggest proportional allocation
across eligible outstanding portions, require review/confirmation, and allow
manual changes. Bank-identified portions take precedence. Partial conversions
use the same default for their converted portions; previously settled portions
are not reallocated. Payments remain transfers counted once; uncertain derived
payment applications also require review. Purchases, exposed fees, refund
credits and opening debt retain their own meanings and category/goal links.
Opening debt is never new spending, and no components or fees are invented.

Confirmed allocation revisions update the affected purchase spending/goal
usage valuation once without automatically increasing goal funding. Preserve
original dates and previous allocations, applying existing review and closed-
budget/current-period correction rules. When composition is incomplete or
uncertain, the actual aggregate conversion can still be recorded while the
purchase-level valuation remains visibly provisional and awaits review.

## Current decision: CLP planning and foreign cost estimates

MVP budget assignments, category/goal funding, goal targets and monthly
contribution plans use CLP. Foreign-currency budget and goal-target choices
are later scope. A planned cost component may nevertheless retain an amount
such as USD 2,000 and show its estimated CLP cost; that component is not a
USD-denominated goal target.

A foreign **cost estimate** is separate from recorded spending and actual
currency balances. For future costs, use a selected current rate and optional
user spread; a selected card-network rate can be used where available, without
promising provider integration or treating it as a guaranteed bank charge.
The user can lock the estimate's selected dated rate or let it follow rates.
Before purchase, these estimates affect forecasts and the estimated funding
gap, not actual spending.

After a foreign purchase, retain the real debt in its actual currency. Record
its initial provisional CLP expense using the established purchase valuation
rule. The user can lock the purchase-day cost estimate or let its cost estimate
vary. A variable estimate changes the displayed funding gap only; it does not
automatically revalue the recorded expense, category/goal remaining amount,
funding or bill reservation. Current net worth independently values actual
outstanding foreign debt at the latest available rate. A lock freezes a
provisional estimate, not the bank's real liability or eventual conversion.

Actual confirmed bank conversion or a reviewed derived aggregate allocation
supersedes the provisional expense under the established source hierarchy.
Apply the difference once to the affected spending/goal usage, availability
and related reservation under existing review, funding and closed-period
correction rules; never automatically increase goal funding. Retain initial
estimates, subsequent cost estimates and actual conversion evidence distinctly.
Significant FX changes govern alerts about the evolving funding gap; the
threshold policy remains open and does not define a new financial amount.

Eligible foreign asset funding follows the automatic revaluation decision
below; it is distinct from this expense-estimate policy.

## Accepted range-display direction: later release, calculation details open

The accepted display direction is to show a range for estimated foreign
backing-resource coverage alongside fixed CLP budget-funded goal progress.
It does not revalue assigned funding, saved totals, target or funded progress.
For example, a CLP 1,000,000 target with USD 500 in the selected expected
backing resources has CLP 450,000–500,000 of scenario coverage at 900–1,000
CLP/USD: 45–50% of the target as coverage, distinct from its funded-progress
percentage. Identify the resources used in the comparison; no money is
reserved or earmarked merely by showing it, and shared backing cannot be
counted as separate funding for every candidate goal.

Foreign future costs can also show an estimated range. Higher FX makes those
costs larger in CLP while increasing the CLP value of held foreign assets.
Compare assets and costs using the same rate scenario consistently, counting
each wallet balance once. These ranges change neither actual balances nor
recorded spending, assignments, goal funding/availability or existing floating
cost-estimate funding-gap behavior.

The FX scenario/range view belongs to a later release. Calculation method,
rate source, historical lookback settings and trend treatment remain open
for that later capability. The agreed floating-cost funding-gap warning
remains in the MVP. User-selected history, such as six months or a
year, and potentially trend-informed ranges are candidate approaches, not
promised calculation methods or provider integrations. Distinguish observed
historical extrema from estimated future ranges; neither is a guarantee that
future rates stay within those values.

## Current decision: split rounding and visible remainders

Use currency-appropriate precision for ordinary entry/display: whole pesos
for CLP and cents for USD. Preserve any higher supplied precision and source
rates for inspection; presentation must not silently alter bank/source facts.
Calculate derived conversions/splits before rounding the resulting portions.
Bank-supplied per-purchase amounts remain authoritative and are not overwritten
by this allocation rule.

Rounded portions must sum exactly to the authoritative total. Distribute any
leftover smallest units to the portions with the largest fractional remainders;
ties follow the displayed portion order used for allocation review. Show which
portion received a remainder unit. Allow manual adjustments that preserve the
total and valid allocation meaning, without editing the authoritative bank
amount. Confirmed allocations do not change merely because a later view is
sorted differently. Apply the same treatment to refunds with their signs
preserved; never invent an extra expense to absorb rounding.

For example, CLP 1,000 divided equally across three portions becomes
CLP 334 + 333 + 333, with the extra peso visible. The analogous USD 1 total
becomes USD 0.34 + 0.33 + 0.33. This rule also applies to rounded derived
allocations from aggregate conversions, reconciling the relevant currency's
total rather than creating another account effect.

Unusual source totals with more precision than the normal allocation unit
remain a precision edge case to refine; preserve their exact supplied amounts.

## Current decision: automatic FX valuation of eligible assets

For the MVP, automatically revalue eligible foreign money held (assets) at
the current available rate for CLP budget funding. Keep the actual foreign
balance unchanged and explain the CLP change with its rate/source/date as a
valuation adjustment, distinct from ordinary income or spending. A positive
FX change increases ready-to-assign once; a negative change reduces available
funding and may produce an over-assigned warning. Assignments, funded goal
amounts, targets and their jobs stay fixed until the user assigns/reassigns
money. No budget lock or forced reassignment follows from the shortfall.

For example, an eligible JPY 5,000 balance was valued at CLP 30,000 and fully
assigned. When its current CLP value becomes 32,000, ready-to-assign increases
by CLP 2,000 automatically; the existing assignment stays CLP 30,000. If its
value instead falls from CLP 30,000 to 28,000, show the CLP 2,000 funding
shortfall without silently reducing that assignment. Current account CLP
estimates update for tracking-only accounts too, but those accounts do not
supply budget funding. Preserve closed snapshots and apply current effects
under the established correction rules, rather than repeatedly counting past
valuation adjustments.

A bank-known converted incoming amount remains an actual amount in the
resulting currency. Foreign wallets/cash retain their real denomination;
using a CLP estimate for eligible held money does not invent a bank conversion.
Confirmed opening/receipt activity and later actual conversions keep their
source and once-only treatment. Count only the FX valuation difference as the
valuation adjustment, not already-recorded receipts, purchases, payments or
owned transfers again as income. Source conversion estimates/final evidence
still follow the existing replacement and review rules.

This automatic policy concerns eligible foreign assets. The foreign expense
cost-estimate choice remains warning-only: it does not automatically revalue
recorded expense/goal usage, category/goal remaining amounts or reservations
before actual bank conversion/confirmed allocation. Current valuation of debt
for net worth stays independent; foreign-debt coverage follows the
warning-only rule in AC-5.17. A future
user choice between automatic and reviewed asset revaluation is later scope;
automatic is the MVP behavior, without adding a wallet-level setting.

## Open first-slice decision: unvalued foreign expenses

Manual foreign-currency expenses retain their original amount/currency. The
first slice excludes transaction-date CLP estimates, but this does not settle
how an entered expense with no confirmed CLP value affects current-month
category activity and availability. Do not silently show it as zero spending;
define whether affected totals are marked unavailable or another reviewable
representation is used before implementing this path.

If Frankfurter cannot be reached, use the latest stored rate and show its
effective date with a stale label. If no rate observation is stored, mark the
foreign valuation unavailable; do not treat the balance as zero or return a
partial CLP budget-funding total.

## Current decision: transfers across the budget boundary

For owned money transfers, two budgetable accounts or two tracking-only
accounts have no budget-boundary funding effect. Independent FX differences
and exposed fees still follow their existing rules, once. Tracking-only to
budgetable adds
funding once; budgetable to tracking-only removes funding once. Explain this
as a separate budget funding effect; the financial movement remains a transfer,
not income or spending. Leave category assignments and funded goals fixed,
showing any resulting over-assignment instead of silently changing their jobs.
No new transfer category or automatic goal usage is needed. Retain real dates
and the existing closed-period correction rules. Payments of already-recognized
debt retain their covered or paid-uncovered treatment under AC-5.19 without
another ordinary boundary deduction for either portion. Unrelated asset
transfers still follow the boundary rule. (AC-5.11)

Example: CLP 100,000 funding, CLP 80,000 assigned and CLP 20,000 ready to
assign. Transferring CLP 30,000 into a tracking-only account leaves
CLP 70,000 funding, CLP 80,000 assigned and CLP 10,000 over-assigned. The money
is still owned; this is not spending. Initial signed balances follow AC-5.13;
positive-credit consumption follows AC-5.18; returns retain transfer treatment. Purchase coverage follows
AC-5.14.

## Current decision: prospective account eligibility changes

Preview the funding effect before confirmation. From the confirmed change
onward, tracking-only to budgetable adds the current eligible positive asset balance once;
budgetable to tracking-only removes it once. Keep assignments and funded
goals fixed, exposing any resulting over-assignment. This is a prospective
funding adjustment, not a transfer, income, expense, category use or goal usage.
Actual balances, bank data, real dates, independent categorization and past
closed budget snapshots remain intact. Do not replay original receipts or
prior valuation adjustments; retain existing foreign-asset valuation rules.
(AC-5.12)

Example: CLP 100,000 budget funding with CLP 80,000 assigned leaves CLP 20,000
ready to assign. Enabling a tracking-only account holding eligible CLP 50,000
adds CLP 50,000 once: funding is CLP 150,000 and ready to assign is CLP 70,000.
No income or physical movement occurred. Debt-bearing changes retain recognized
debt and protection under AC-5.19.

## Current decision: actual signed budgetable opening balances

Actual negative bank/cash/savings balances reduce initial budget funding;
actual positive card credits add it when budgetable. Unused credit and
overdraft limits provide no money. Budgetable opening balances of
CLP 100,000 in one bank, CLP -20,000 in another and CLP 5,000 of actual card
credit provide CLP 85,000 before assignments. Preserve opening card-debt
treatment and count later transactions/purchases/reservations once, without
subtracting debt twice or treating positive card credit as automatic new
income. Eligibility still controls budget participation. Positive-credit
consumption follows AC-5.18; debt-bearing eligibility changes follow AC-5.19.
(AC-5.13)

## Current decision: partially funded card purchases

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

## Current decision: partial card payments

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

## Current decision: card-refund coverage

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

Starting with a CLP 1,000 purchase funded with CLP 600 in its original
category, CLP -400 original-category availability, CLP 600 reserved and
CLP 400 uncovered debt, a linked
CLP 300 refund leaves debt CLP 700, category availability CLP -100, reservation
CLP 600 and uncovered debt CLP 100. No reservation is released yet.
Starting instead from that same original state, a CLP 500 refund leaves debt
CLP 500, category availability CLP 100, reservation CLP 500 and no uncovered
debt. Both cases reduce spending once without income or new ready-to-assign.

For example, with CLP 600 genuinely available in a category, a CLP 1,000
card purchase records CLP 1,000 spending and debt, leaves category availability
at CLP -400, reserves CLP 600 for the bill and shows CLP 400 uncovered debt.
Before any payment, a later explicit CLP 400 assignment/reassignment backed
by eligible money brings category availability to zero and the reservation to CLP 1,000;
spending and debt remain CLP 1,000. Merely entering an unsupported CLP 400
assignment does not cover that debt or increase the cash-covered reservation.

## Current decision: foreign-debt valuation and coverage

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

For a separate case, a budgetable bank holds CLP 100,000. A USD 100 purchase
initially valued at 900 CLP/USD has CLP 90,000 spending and genuinely funded
reservation, leaving CLP 10,000 ready to assign. At 950 CLP/USD, debt remains
USD 100 and current liability valuation is CLP 95,000. Show an estimated
CLP 5,000 additional coverage gap; spending/reservation stay CLP 90,000 and
ready-to-assign stays CLP 10,000. If CLP 30,000 was already uncovered instead,
show CLP 35,000 total estimated shortfall: CLP 30,000 original plus CLP 5,000
FX difference, not another CLP 35,000 on top of the original shortfall.

On an actual CLP 95,000 conversion of the fully reserved case, apply the
CLP 5,000 actual expense difference once, replacing the estimated gap. Do not
add both the estimate and actual difference. With funding unchanged, category
availability becomes CLP -5,000 and genuine reservation remains CLP 90,000
until explicitly covered. If the user then assigns CLP 5,000 of available
funding, ready-to-assign becomes CLP 5,000, category availability zero and
reservation CLP 95,000, without another expense or debt increase.

## Current decision: spend actual positive card credit

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

## Current decision: eligibility changes preserve recognized debt

Making an indebted card or negative-bank account tracking-only stops its
positive eligible assets from funding prospectively but does not erase debt
already represented by the budget. Retain its existing protection,
reservations or uncovered status until settled, without releasing
ready-to-assign or undoing category spending. Enabling participation recognizes
only debt not already represented once under opening-debt rules, preserving
prior coverage and showing shortfalls rather than inventing funded reserves.
A payment of already-recognized debt keeps its once-only treatment even
after the target becomes tracking-only: covered portions consume protection;
uncovered portions keep the paid-uncovered gap treatment of AC-5.15. Neither
portion also incurs an ordinary boundary-outflow deduction. Unrelated asset
transfers and positive-credit portions retain their existing treatment. Changes remain previewed and prospective with
warnings rather than hard limits; actual balances, source data, real dates,
independent reports and closed snapshots remain intact. (AC-5.19)

Example: bank CLP 100,000 less already-recognized/protected CLP 95,000 for
USD 100 card debt leaves CLP 5,000 ready to assign. Making the card
tracking-only leaves that CLP 5,000 and protection unchanged; its later covered
payment consumes the protection once. Foreign-debt FX and actual conversion
rules remain as agreed, independently of the eligibility setting.

## Current decision: ordinary goal-purchase refunds

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

## Current decision: explicit goal funding reassignment

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

## Current decision: goal links on purchase portions

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

## Confirmed direction

This section is an earlier discovery record. The current decision block above
supersedes its statements about zero-based budget assignment, pool
reservations, and greedy or paced allocation of reserved goals; other
discovery points remain historical context where they do not conflict.

- Initial user: one person managing their own finances.
- Main problem to solve: understand where their money went.
- Budgeting style: zero-based budgeting is a core product direction. Available
  money is assigned in the budget to goals or spending categories; assignments
  retain their purpose across periods without creating new funding.
- Audience and conventions: Chile-first, with future Open Finance as a possible
  way to organize financial activity.
- Lukas organizes financial information and reads statements; it does not move
  money or initiate payments or transfers.
- The user wants named groups of accounts or money with a shared purpose. For
  example, a `general` pool might include bank A and cash; a `savings` pool
  might include savings A and investment B; an `emergency` pool might include
  savings B and investment C.
- Long-term, an account may contribute to multiple pools by full balance,
  percentage or fixed amount, with a remainder assigned elsewhere.
- First-release pools group whole accounts: an account's full balance is
  assigned to one pool. Partial allocation rules come later.
- Future partial pool allocations may either follow balance changes
  automatically or remain fixed until the user adjusts them. Earlier proposals
  for pool views included amounts available after goal reservations; those
  reservation-based figures are superseded by the current decision above.
  Pool views may also show a separately user-set amount the user considers
  spendable.
- The user chooses per pool whether the user-set spendable amount is
  display-only or the amount used for planning.
- Future partial allocation rules may have user-configurable order and allow
  percentages to use either the full account balance or the remainder after
  earlier allocations.
- Pools include money-holding accounts such as bank, cash, savings and
  investments. Credit-card accounts can be associated with a pool for their
  transactions, but their liability/available-credit balance is excluded from
  pool balances.
- Future partial allocations may have user-defined shortfall behavior if the
  account balance falls below an intended allocation.
- Pools spanning currencies should show both a CLP total and the breakdown in
  each original currency. The CLP total is a current estimate using the latest
  available rate, with the rate and date shown.
- For non-CLP transactions, use the final CLP amount from the statement when
  available; otherwise use a transaction-date estimate and show its source and
  rate.
- The first release should include a simple version of pools; advanced
  allocation rules can follow later.
- The first version should support both manual transaction entry and file
  imports.
- The first spending report should default to the current calendar month.
- Imported transactions should affect reports only after the user reviews and
  confirms them. This is a current preference that may change later.
- A pending card estimate counts in spending immediately; when the posted
  amount arrives, it updates the spending value while preserving the pending
  and posted context.
- The first version should support both CSV and Excel imports.
- Initial account types include bank, cash, credit card, savings and investment
  accounts.
- Regular account balances should start from an opening balance and be
  calculated from confirmed transactions. Users can reconcile calculated
  balances against real balances.
- Users can start an account with an opening balance as of a date without
  importing its full prior transaction history.
- Opening balance date defaults to today, with an option to choose another
  date.
- Reconciliation should show a difference first and let the user resolve it
  when ready. The user can review and match account activity, create an
  explicit adjustment for an unexplained difference, or leave reconciliation
  unfinished. A reconciliation adjustment is a distinct, labeled account
  entry; it corrects the account balance, shows its effect on available-to-
  assign, and is not treated as ordinary category spending. Its source and
  history remain visible. Any account type may be reconciled if the user wants
  to.
- Spending categories should include built-in options that the user can edit,
  plus categories the user creates.
- Expenses may be saved or confirmed without a category; reports show them as
  uncategorized until the user assigns one.
- Import review should support both reviewing individual transactions and
  confirming selected transactions in bulk.
- Likely duplicate import rows are held for an individual decision outside
  bulk confirmation. The user can keep them, link/merge them with existing
  activity, or discard them.
- Investment accounts should track account value, its variation from the prior
  period, and activity such as user deposits and interest. Holdings-level
  tracking is not currently part of the described need.
- Goals and projections are expected in the first release.
- Earlier proposal, superseded above: both unreserved `goal to buy` and
  reserved saving goals belong in the first release, including greedy and
  paced reservation strategies.
- A goal should have a target amount and date, track contributions and show
  progress. A fuller goal definition can follow later.
- A goal is associated with one financial pool and works through that pool,
  rather than being linked directly to accounts.
- Earlier proposal, superseded above: goals should support both unreserved
  `goal to buy` behavior, where goals compete for pool money, and reserved
  saving goals, where an amount is set aside for a specific goal within its
  pool.
- For goal-to-buy goals sharing a pool, show whether each target is affordable
  on its own and warn when the combined targets exceed that pool's available
  money. Do not automatically pick a winner or reserve money for these goals.
- If the user-set spendable amount is selected for planning, show goal
  affordability against both calculated pool availability and the user-set
  amount. Neither amount limits the goal; the difference is an informational
  planning signal.
- Show an informational total of the target amounts for all goals linked to a
  pool. This total includes both goal-to-buy and reserved goals; it does not
  change pool availability or allocate money. Treat this display rule as
  provisional for later review.
- Earlier proposal, superseded above: reserved goals can use priority
  allocation, with greedy filling the highest-priority goal first and paced
  reserving only the amount needed for that period to stay on track.
- Goal reservations are virtual bookkeeping that changes goal and pool
  availability only; Lukas never moves money between accounts.
- Earlier proposal, superseded above where it describes reservation
  allocation: allocation behavior can be chosen per goal. A paced goal can use
  either a target date or a user-set monthly amount.
- Goal usage can be linked manually or from a product suggestion the user
  confirms.
- Automatic goal suggestions may use any useful transaction clues, but they
  must be confirmed by the user; this capability can come later. Manual goal
  linking is in the earlier scope.
- A goal may have no target date (a general amount to work toward) or a specific
  target date. For dated goals, Lukas should show how much to save each period.
- Spending associated with a goal should be tracked as `used` without
  automatically reducing the goal's recorded `saved` amount. The user expects
  a transaction to reduce the goal value only in exceptional cases.
- When linking a transaction to a goal, default to ordinary usage. The user
  can explicitly mark an exceptional transaction to reduce cumulative saved
  instead; in that case saved decreases by the transaction amount rather than
  used increasing. Existing used remains visible and availability is
  recalculated.
- Goals should show saved, used and available amounts separately. For the
  first release, progress is saved divided by target; other progress formulas
  may be considered later.
- Required goal savings should be expressed per calendar month.
- The first projection should show how current cashflow may look over the next
  months.
- The first projection should use recent income and spending patterns, and the
  user should choose the number of months projected.
- The user chooses the historical lookback period for each projection.
- Projections should show estimated income, spending, net cashflow and progress
  toward dated goals.
- Goal savings should reduce money projected to remain after expenses, while
  staying separate from spending.
- Simple recurring expenses belong in the first release. Each occurrence is a
  forecast estimate whose amount can be adjusted independently; a confirmed or
  imported actual transaction can be linked to it.
- Recurring rules can have end dates, and individual occurrences can be
  skipped or rescheduled.
- For each projection, the user chooses whether recurring rules or
  history-based estimates take precedence when both estimate the same expense.
- When a scheduled goal purchase overlaps a recurring or history-based
  estimate, the user decides whether they describe the same cost or different
  costs. Until marked as the same cost, keep both separate and count both; a
  linked same-cost pair counts once. Keep both sources visible and do not infer
  a match.
  If the user marks a pair as the same cost but estimates differ, which amount
  drives the projection remains open.
- Multi-transaction recurring event templates, such as a birthday trip with
  separate gas, toll and gift expenses, come later.
- The first release should include simple whole-account pools. Advanced
  partial account allocation rules may come later.
- Investment account value should be included early; investment activity such
  as deposits and interest may follow later.
- Each CSV or Excel import file represents one account for the initial scope;
  this restriction may change later.
- The first release is a web application. The CLI comes in a later phase.
- The first release uses manual entry and CSV/Excel; Open Finance connections
  come later.
- Confirmed transactions should be corrected by editing the transaction
  directly. The original details and subsequent changes must remain
  inspectable. The correction preserves the actual transaction date and
  corrects account/report history. If it affects a closed budget, preserve
  that budget's close history and apply the budget impact as an adjustment in
  the current period where possible.
- Unconfirmed imported rows should remain in a review list for later action.
- For the first spending view, the user wants both category and account
  breakdowns.
- A transaction may have one category or be split across categories. Categories
  describe what the spending was for; tags describe why or its cross-cutting
  context (for example, category `transport` and tag `grandma-visit`).
- Credit-card purchases count as spending when made; paying the credit-card bill
  is a transfer. The user may want a purchase or payment assigned to a later
  reporting period.
- In the zero-based budget, a credit-card purchase uses its category budget
  when made and virtually reserves only genuinely cash-covered budgeted money
  for the later bill (AC-5.14). The bill payment is a transfer without another
  assignment for that covered money.
- Each budget lets the user choose its period basis: actual transaction date
  or assigned reporting period. Card purchase budget impact and the associated
  bill reservation follow that budget's selected basis; the actual transaction
  date remains intact.
- Refunds should reduce spending in the original category.
- If a refund has no obvious original transaction, the user can categorize it
  and it reduces that category's spending.
- A separately exposed fee defaults to the related purchase's category.
- Transactions may have multiple user-created tags.
- Support both per-card statement-period rules and manual period assignment.
- Store each card's statement closing day and payment due day. Use the due day
  mainly as a reminder when the bill has not been paid on time; Lukas does not
  initiate payments. Use the statement amount due when available and allow the
  user to correct it; otherwise calculate it from the card's recorded activity
  through the statement close. Clear the reminder when recorded payments cover
  the amount due, or let the user mark it as read without changing payment
  status.
- Add a split engine for shared expenses with Splitwise-like behavior and a
  way to work with Splitwise. The native feature should be able to read and
  write split data to Splitwise when the user chooses it as the backend. Shared
  expense tracking is after the first release.

## Working first-release scope

This is a synthesis of the user's confirmed scope so far, not a frozen MVP
specification. Details and open questions remain below.

**Historical/superseded pool-reservation material:** Some use-case drafts and
open questions below describe pool reservations or separate pool and budget
funding. Keep them as discovery history only; they are superseded by the
current decision at the top of this document. In the current model, budgets
fund specifically funded goals and shared savings categories; pools show
expected payment sources.

### Understand monthly spending

- Track bank, cash, credit-card, savings and investment accounts. Investment
  accounts start with basic account value; transaction-level investment
  activity can follow later.
- Start accounts from an opening balance and date; calculate balances from
  confirmed transactions and support reconciliation.
- Enter transactions manually or import CSV/Excel, one account per file.
  Imports support individual and bulk review; unconfirmed rows stay out of
  reports and remain available in a review list.
- Provide built-in editable categories, user-created categories and multiple
  user-created tags. A transaction can split across categories. Categories
  describe what; tags describe context or why. Expenses may remain
  uncategorized and appear in an uncategorized report group.
- Show a current-calendar-month report by category and account. Credit-card
  purchases count as spending; card bill payments are transfers. Refunds
  reduce spending in their category.
- Convert non-CLP spending using the final statement CLP amount when available;
  otherwise use a transaction-date estimate with its source and rate shown.
- Support per-card statement-period rules and manual transaction period
  assignment while retaining each real transaction date. Each report can
  choose whether to use the assigned period or the real date; for card
  statements, it can also choose the close-date or payment-date period.
- Configure a recurring statement closing day and payment due day per card;
  use the payment due day mainly to remind the user if payment is late. Lukas
  does not initiate card payments.

### Build a zero-based budget

Zero-based budgeting is a core product direction, and its basic first-release
flow is defined. Some edge decisions remain open. Use calendar months in the
first release, matching the default monthly spending view. The user wants to
assign all money available
in accounts plus income received during the period to spending categories and
goals. Pools continue to group accounts; they are not budget assignment
destinations. At period close, the user chooses per category or goal whether
unspent money carries forward or returns to the unassigned amount. Carried
money remains assigned next period and is excluded from new unassigned money,
even though it remains in account balances. Available-to-assign starts with
eligible money not already assigned at the period start, plus each income
receipt once. The user chooses per account whether its balance is eligible for
budgeting; an income receipt is not counted again because it is reflected in
the updated account balance. A partially assigned budget can be saved and
shows the amount waiting for a job. An over-assigned budget can also be saved
and shows the shortfall as `over-assigned by [amount]`.
Existing credit-card debt reduces money available to assign; credit limits
never count as money. New card purchases use their category budget once and
virtually reserve its genuinely cash-covered portion for the later bill
(AC-5.14); paying it is a transfer without another assignment for covered
money. Specifically funded goals receive ordinary budget assignments; goal-to-buy purchases use a shared budget
category. Greedy and paced allocation apply to budget assignments, not pools.
A budget period remains open until the
user explicitly closes it; while open, transactions dated in that period can
still be added or edited even after the calendar month ends. Closing includes
reviewing carryover choices. A closed period can still be corrected through a
more deliberate path. Correct the transaction directly while preserving its
actual date and history. Preserve the closed budget snapshot and, where
possible, apply the budget impact as an adjustment in the current period.

### Organize money with simple pools and goals

- Include whole-account pools in the first release. Each money-holding
  account's full balance belongs to one pool. Credit-card transactions may be
  associated with a pool, but the card balance is excluded from pool balances.
  Partial account allocations and their percentage/fixed-amount rules come
  later.
- For multicurrency pools, show original-currency amounts and a current CLP
  estimate with the latest rate and its date.
- Show pool holdings and expected payment-source coverage separately from
  budget assignments. Calculated and user-set spendable amounts remain
  distinct planning signals; pools do not reserve or fund goals.
- Include specifically funded goals as budget categories with targets and
  goal-to-buy candidates that use a shared budget category. Specifically
  funded goals track saved, used and available; greedy and paced allocation
  apply to budget assignments. Progress is saved divided by target. Manual
  goal links on purchase portions are included; automatic confirmed suggestions can
  follow later.

### Project cashflow

- Estimate future months from recent income and spending patterns. The user
  chooses the lookback period and projection horizon.
- Show estimated income, spending, net cashflow and progress toward dated
  goals; subtract goal savings from projected money left after expenses, but
  keep it separate from spending. Keep estimates distinguishable from actuals
  and explain their basis.
- Include simple recurring-expense estimates; allow an amount adjustment for
  each forecast occurrence, support end dates and per-occurrence skip or
  reschedule, and link confirmed actuals to the relevant estimate. Let the user
  choose per projection whether explicit rules or history-based estimates
  apply when they overlap. For an overlap between a scheduled goal purchase and
  an estimate, let the user mark them as the same cost (count once) or separate
  costs (count both), retaining both sources and never inferring the match.

### After the first release

- Native shared-expense tracking, including one-time outings, household groups,
  settlements and optional Splitwise read/write.
- Investment activity such as deposits and interest.
- Automatic transaction-to-goal suggestions.
- Partial account-to-pool allocation rules.
- Multi-transaction recurring event templates.

### Other product capabilities to place

- A CLI for people and agents to query and add data is part of the accepted
  rewrite direction and is planned for a later phase; its user-facing
  workflows remain open.
- Open Finance is a future opportunity in the initial idea and is planned for
  after the first release. Lukas does not initiate payments or move money.

## Candidate use case: Understand spending over a period

**Actor:** An individual managing their finances.

**Goal:** See where money went during a selected period.

**Draft flow:** The user records or brings in financial activity, reviews any
uncertain or incorrect details, and views spending summarized by a useful
classification. The user can inspect the underlying transactions so totals can
be understood and corrected.

**Current rules:** Credit-card purchases count as spending when made; card bill
payments are transfers. Refunds reduce spending in their category. A
transaction can have a single category or be split across categories. Tags add
cross-cutting context (the category says what; a tag can say why, such as
`transport` tagged `grandma-visit`); users can apply multiple tags and create
their own. Every transaction requires an account, date, amount and type
(income, spending or transfer). A transfer also requires the other account.
Payee, category and note are optional; uncategorized activity remains
available for review in reports. The current default period is the calendar
month, and the initial data-entry paths are manual entry and CSV/Excel import.

**Refund example:** A CLP 40,000 grocery purchase and a CLP 8,000 refund in the
same reporting period produce CLP 32,000 net grocery spending. The original
purchase and refund remain separately inspectable.

An unlinked refund can be categorized by the user and reduces spending in the
selected category. A fee exposed separately in a statement defaults to the
related purchase's category while remaining reviewable as source detail.

## Candidate use case: Import and review transactions

**Actor:** An individual managing their finances.

**Goal:** Bring statement data into Lukas without letting unreviewed rows
change financial reports.

**Draft flow:** The user imports a CSV or Excel file, reviews transactions
individually or selects transactions for bulk confirmation, and confirms the
rows that should become part of their financial activity. Confirmed rows then
appear in reports. Rows not confirmed do not affect reports or balances. Rows
missing required values or with an unclear transaction type are flagged in
review; the user can correct them before confirmation, and the original
imported values remain inspectable.

**Current direction:** Each file represents one account for the first release;
unconfirmed rows stay available in a review list. Incomplete or ambiguous rows
are flagged and correctable before confirmation, with their original imported
values preserved for review. No transaction fields are locked during this
review; the row remains associated with the account selected for its file.

**Open product questions:** How duplicate and malformed rows appear in review.
Likely duplicates are held for individual decisions: keep, link/merge, or
discard. Pending card estimates count immediately and are updated by posted
amounts while preserving their review context.

## Account scope to define

Initial account types named by the user are bank, cash, credit card, savings
and investment accounts. Basic investment account value is expected early;
investment activity such as deposits and interest may follow later. The current
need does not include holdings-level detail. Define how account values are
updated and what comparison period means before planning this use case in more
detail.

## Candidate use case: Start and reconcile an account

**Actor:** An individual managing their finances.

**Goal:** Begin tracking an account without entering all past activity, then
keep the calculated balance consistent with the institution or a statement.

**Draft flow:** The user creates an account with an opening balance as of a
chosen date. Confirmed transactions after that date change the calculated
balance. The user compares that balance with a real balance and reconciles any
difference.

**Current direction:** Opening balance date defaults to today and can be
changed. Reconciliation first presents the difference and lets the user
resolve it later. Any account type can be reconciled if desired. The user
chooses the resolution process each time.

**Open product questions:** Which resolution actions should be offered, and how
should adjustments and decisions be explained in account history?

## Candidate use case: Track a financial goal

**Historical draft, superseded by the current decision at the top of this
document.** The pool-reservation and automatic cross-pool funding behavior in
this section records an earlier exploration. Current behavior uses budget
assignments to fund specifically funded goals; a purchase is categorized to
the goal and does not create funding based on its payment account or pool.

**Actor:** An individual managing their finances.

**Goal:** Work toward a financial target and understand progress over time.

**Draft scope:** A goal has a target amount and is associated with one
financial pool, not directly with accounts. It may have no target date or a
specific target date. For a dated goal, Lukas shows how much the user needs to
save each calendar month. Goals support two behaviors:

- **Goal to buy:** No amount is reserved. The goal indicates a possible use of
  its pool's funds. Goals using the same pool compete for available money, and
  Lukas should make clear when each goal is individually affordable but the
  pool cannot fund all of them at once.
- **Reserved saving goal:** An amount is set aside for a specific goal within
  its pool as virtual bookkeeping, without moving money between accounts. The
  user can set goal priority and choose a greedy allocation that fills the
  highest-priority goal first, or a paced allocation that reserves only what
  is needed for the current period to stay on track, leaving the rest available
  to other goals. This behavior is selected per goal. A paced goal can use a
  target date or a user-set monthly amount.

A purchase such as a plane ticket can be associated with the goal as usage.
How it affects saved and available amounts depends on whether the transaction's
account is in the goal's pool, as described below. The actual transaction
always affects the real account balance; a card liability is not pool money.

**Cross-pool purchase rule:** A purchase can be linked to a goal even when it
is charged to an account outside the goal's pool. If a goal has 5,000 saved
from its pool and a 1,000 goal-related purchase is charged to a credit card
outside that pool, add 1,000 to both cumulative saved and used. Available
(saved minus used) remains 5,000, because the goal pool's money was not touched.
The card liability increases, but is not counted as pool money. The later
card-bill payment is a transfer, not a second expense or goal use. If a
goal-related purchase is charged to an account in the goal's pool, increase
used but do not increase saved; available then decreases by the purchase amount.

In the user's example, the `vacations` goal has a 10,000 target, 5,000
allocated from a `savings` pool containing 7,000, and a 1,000 ticket is charged
to a card associated with `general`. The goal then shows 6,000 saved, 1,000
used and 5,000 available. The virtual reservation from the savings pool remains
5,000, and that pool remains at 7,000 until money is actually spent from it.
The card's liability is tracked on the account but is not included in the
pool's available money.

Transactions can be linked to goal usage manually or through a suggestion the
user confirms. Goal usage is a dedicated transaction field; tags remain
separate and describe context such as `transport` or `grandma-visit`. A
`$goal-a` tag-like prefix is not used to represent goal usage.

**Example from discovery:** For a 5,000 goal with 4,000 saved, a 1,000 ticket
purchase should display target 5,000, saved 4,000, used 1,000. The purchase
must not rewrite saved as 3,000.

**Visual example from the user:** For a 10,000 goal, the progress bar shows
1,000 used, reaches the 5,000 saved marker, and leaves the rest to the 10,000
target unfilled. Confirmed interpretation: 5,000 is the cumulative saved
amount, including the 1,000 used; 4,000 of that remains available. `Used` is a
portion of the accumulated `saved` amount, not an additional amount to add to
it. For a reserved goal, available is saved minus used. The progress
calculation remains selectable per goal; this example marks progress at the
cumulative saved amount.

**Current direction:** Show saved, used and available separately. In the first
release, progress is saved divided by target; ordinary used funds remain part
of saved and do not reduce that progress. An explicit exceptional transaction
reduces saved instead of increasing used; existing used remains visible. The
required savings amount is per calendar month. Other progress formulas may be
considered later.

Credit-card transactions may be associated with a pool, but their liability
and available-credit balances do not count as pool money.

**Pool competition example:** If a pool has 4,000 available, unreserved goals A
(3,000) and B (2,000) may each be individually affordable, but completing both
would require 5,000. The product should communicate that the pool can fund only
one of them at that time. A reserved goal instead sets aside its allocated
amount, reducing what is available to other goals.

## Candidate use case: Project upcoming cashflow

### Workbook clarification: planned purchases and pool spending order

The user's savings workbook distinguishes current grouped account balances,
planned savings contributions, purchase costs and reserve targets. The pool /
goal funding relationship is being reconsidered; earlier pool reservations
and separate pool/budget goal assignments are not a settled final model.

Confirmed planning behavior:

- A scheduled goal purchase reduces projected savings in its planned period.
  Subsequent periods carry that reduced balance forward.
- If its preferred pool is insufficient, automatically cover the shortfall
  from subsequent pools in the user's chosen spending order. Show the amount
  drawn from each pool.
- These are projection calculations, separate from monthly budget assignments
  and actual account transactions.
- Reserve targets measure desired retained resources; they are not scheduled
  purchase costs. They are warning thresholds: fallback may draw below them,
  and the projection shows the resulting shortfall against each target.
- **Later scope:** The user can mark a pool and/or account as restricted and define events
  that unlock it for planning. The projection excludes restricted resources
  until their user-defined unlocking conditions are met. Keep total holdings
  distinct from resources eligible for the scenario. External processes for
  actually accessing funds are outside scope; no provider-specific unlocking
  requirement is implied by the workbook example.
- Once eligible resources are exhausted, stop simulated drawdowns at zero
  and show the remaining cost as an unfunded shortfall. Restricted account
  balances cannot cover that shortfall unless their availability condition is
  met. Pool and account restrictions apply together: an account is eligible
  only when both its own conditions and its pool's conditions are satisfied.
  Unlocking either one does not override the other's restriction.
- The user manually activates unlocking events within each projection and
  chooses their effective dates. A resource becomes eligible from that date
  once all applicable account and pool conditions are satisfied. These events
  are scenario assumptions, separate from actual accounts and monthly budgets.

Workbook example: December's projected savings of CLP 1,762,655 less a planned
CLP 350,000 purchase leaves CLP 1,412,655 to carry into January. The workbook's
E88 carrying D90 instead of D92 was confirmed to be unintended; the source
workbook has not been edited.

**Actor:** An individual managing their finances.

**Goal:** See how current cashflow may look over the next months.

**Draft scope:** The user views an estimate of upcoming cashflow based on
recent income and spending patterns. The user chooses the number of months to
project and chooses the historical lookback period for each projection.
Projected values remain distinguishable from actual transactions and should be
explainable. Results show estimated income, spending, net cashflow and progress
toward dated goals. Goal savings reduce the projected money left after
expenses, but remain separate from spending. Projection assumptions and
comparison options remain open.

## Candidate capability: Recurring expenses

The user wants a simple recurring-expense flow and a more advanced recurring
event flow. Recurrence rules should support:

- Every N days.
- On a selected day of each month.
- At the end of each month.
- On an ordinal weekday, such as the last Friday of the month.

The advanced flow defines a recurring event with multiple expense components.
For example, a birthday celebration might include a trip to another city with
separate gas, toll and gift expenses.

**First-release direction:** Simple recurring expense rules generate forecast
estimates. Each occurrence can have its own estimated amount, and a confirmed
or imported actual transaction can be linked to it. Multi-transaction event
templates are later scope.

**Current direction:** Support end dates and skipping or rescheduling individual
occurrences. For each projection, the user chooses whether recurring rules or
history-based estimates take precedence when both estimate the same expense.
Separately, when a scheduled goal purchase and a history/recurring estimate may
describe the same cost, the user decides whether they are the same or different
costs; count a same-cost pair once and different costs twice without inferring
the overlap.

**Current link behavior:** Lukas may suggest which actual transaction matches a
forecast occurrence, but the user confirms the link. Once linked, the actual
replaces that occurrence's estimate in the projection; both remain
distinguishable in the activity history.

## Candidate capability: Split shared expenses

The product should include a split engine with behavior comparable to
Splitwise. Users may choose Splitwise as a backend for shared-expense data, in
which case Lukas should be able to read and write that data. Shared-expense
records and settlements are data workflows; Lukas does not move money or
initiate payments.

**Draft reporting rule:** If the user pays CLP 30,000 for a shared dinner and
their own share is CLP 10,000, the account records the CLP 30,000 outflow and
personal spending reports count CLP 10,000. The CLP 20,000 owed by others is
tracked separately and does not count as money the user has while unpaid.

Shared expenses should cover both directions: the user pays and is owed by
others, or someone else pays and the user owes their share. Initial split
methods should include equal shares, custom amounts, percentages, and
item-by-item splitting.

**Current direction:** The user can mark an amount owed as paid, either linked
to an actual account transaction or without that link, depending on the
settlement. A repayment should not be counted again as income or spending.

### Candidate use case: Split shared expenses and settle up

**Actor:** The user and one or more named participants, with optional Lukas
collaborators.

**Goal:** Track expenses paid by different people, understand each person's
share, and settle balances for either a one-time event or a recurring group
such as a household.

**Inferred flow from discovery:** A shared expense records who paid, its
category, and how it is divided among participants. The user can record
expenses they paid and which people they paid for, as well as expenses someone
else paid on their behalf. Itemized purchases can be split across categories
and participants. Shared expenses can belong to an ongoing household group or
to a one-time outing; a persistent group is not required for every split. The
user's spending reports include their own share in the relevant category even
when someone else paid. Amounts owed in both directions can be offset, so
participants may settle only the net balance. The user may mark that
settlement paid either by linking an actual account transaction or without
that link, depending on the settlement. A repayment does not count again as
income or spending.

**Category recommendation:** Put categories on the underlying shared expense
(or its itemized parts), not on the settlement. Link a repayment to the
expense or net settlement it closes. This keeps the user's share categorized
and the repayment traceable without counting the same purchase twice.

**Release timing:** Shared-expense tracking follows the first release.

**Open details:** Whether partial settlement is supported; whether balances
can net only within a particular group/event or across groups with the same
person, selected per settlement; whether the netting period is configurable; what access optional
collaborators receive to shared and personal information; how the Splitwise
backend keeps records in sync; and which split features belong in the first
release.

## Candidate capability: Purpose-based pools

Pools are user-named groupings associated with accounts or portions of their
balances and a shared purpose. First-release pools assign an entire account to
one pool. Later, the user wants an account to contribute portions to multiple
pools using percentage and fixed-amount allocations, including assigning the
remainder to another pool.

## Candidate use case: Review pool allocations

**Historical/superseded detail:** The draft flow and pool-view choices below
record earlier exploration. The current decision at the top of this document
defines pools as expected payment sources and budget assignments as the source
of goal funding; the pool reservation and user-set spendable-amount details
below are not current decisions.

**Actor:** An individual managing their finances.

**Goal:** Organize money by purpose without confusing those purposes with
separate real-world accounts.

**Draft flow:** In the first release, the user creates purpose-based pools and
assigns whole accounts to them. The user reviews assigned and spendable
amounts, including a CLP total and original-currency breakdown for pools
spanning currencies. Later, the user can split account balances among pools
with fixed or balance-following percentage rules and remainder handling.

**First-release scope:** Whole-account grouping for money-holding accounts.
Credit-card transactions may be associated with pools, but card balances are
excluded from pool totals.

**Later scope:** Partial allocations, percentages against full or remaining
balances, configurable order, and per-allocation shortfall behavior.

**Historical draft direction (superseded):** Pools are expected payment sources
and do not reserve goal funding. Show calculated pool holdings and a separate
user-set spendable amount. The user chooses per pool whether the user-set
amount is display-only or drives planning.

The pool CLP total should use the latest available rate as a current estimate,
show the rate and date, and retain the original-currency breakdown.

**Historical pool-view proposal (superseded):** Show total pool holdings and
the separate user-set spendable amount. Budget assignments to goals are shown
independently and do not reduce or reserve pool holdings.

## Candidate use case: Review credit-card spending by period

**Actor:** An individual managing their finances.

**Goal:** Understand purchases in the period they should affect while retaining
the real dates and avoiding counting the later card payment as spending again.

**Draft rules:** A purchase is spending on its transaction date by default;
the payment from a bank account to the card account is a transfer. The user may
assign a purchase or payment to a later reporting period. The real transaction
date remains visible and unchanged by that assignment. This supports users
whose card cycle resets on a date such as the 25th and whose payment occurs on
the 1st of the following month.

**Current direction:** Support both per-card statement-period rules and manual
period assignment. Users may override the rule for an individual transaction.
The actual transaction date remains available independently.

**Current direction:** Each report can choose whether to use the assigned
period or the actual transaction date.

**Current card-period direction:** Each report can use the statement close date
or payment due date to determine the period.

**Reminder direction:** Configure the statement closing day and payment due
day for each card. The due day primarily reminds the user when a bill has not
been paid on time; Lukas does not make payments. Use the statement amount due
when available and allow correction; otherwise calculate it from recorded
account activity through the closing day. Clear the reminder when recorded
payments cover that amount, or let the user mark it as read without changing
payment status.

## Product boundary to explore

**Historical/superseded exploration:** The boundary below preserves an earlier
pool-allocation discussion. The current decision at the top of this document
defines budgets as the source of goal funding and pools as expected payment
sources; references below to pool reservations or pool funding are not current.

The earlier proposal was to keep actual account balances, purpose-based pool
groupings and budget assignments understandable as separate views. It assigned
whole money-holding accounts to one pool in the first release, with partial
account allocations later. The current decision at the top of this document
settles that budgets fund goals while pools indicate expected payment sources;
there is no separate pool reservation or funding amount.


## Actual Budget patterns adopted for product refinements

The following refinements resolve the earlier open account/import/report
questions. Net-worth reporting joins the MVP; the user deferred all exports.

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
eligibility and debt rules.

Before review, select the account and, for Excel, the worksheet containing the
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
meaning. Missing category, payee or note does not block confirmation.

The MVP includes a current net-worth snapshot: account assets less liabilities
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
and all exports are deferred beyond the MVP.

Actual documents account names, budget eligibility and starting balances in
[Accounts](https://actualbudget.org/docs/accounts/) and
[Starting fresh](https://actualbudget.org/docs/getting-started/starting-fresh/).
The required base-currency/type fields and eligibility defaults above are Lukas
adaptations; budget exclusion does not disable Lukas categorization.

The [Actual import guide](https://actualbudget.org/docs/transactions/importing/)
supports the mapping-preview pattern, date formats, delimiter/sign controls
and separate inflow/outflow columns. Excel worksheet selection and the staged
per-row correction queue are Lukas adaptations preserving our review rules.

[Actual reports](https://actualbudget.org/docs/reports/) inform net worth and
cashflow views. [Transaction filters](https://actualbudget.org/docs/transactions/filters/)
inform inspection and filtering; Lukas requires matching split portions only
in filtered totals. Current snapshot valuation and precise comparison handling
above are Lukas refinements, not claims about Actual validation behavior.
