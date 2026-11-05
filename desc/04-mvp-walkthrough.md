# MVP walkthrough: one month in Lukas

Status: Working product scenario based on `03-spec-v1.md`. The example is for
reviewing feature behavior and scope, not a final design. Sample amounts are
illustrative.

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
  used.

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

## User and starting point

One person in Chile wants to understand where their money went this month and
how much will remain after expected expenses and goal savings, while assigning
available money a job through zero-based budgeting.

The user has:

- A bank account with a dated opening balance.
- A credit card whose transactions are associated with the `general` pool;
  its liability or available credit does not count as pool money.
- Savings accounts A and B with balances totaling 7,000, assigned in full to a
  `savings` pool.
- A `vacations` goal linked to `savings`, with a 10,000 target, a target date
  twelve months away, and 5,000 assigned through the budget.
- A simple monthly internet expense rule with an estimate that can vary by
  occurrence.

Initial goal funding is assigned from eligible opening money ready to assign.
A 5,000 budget assignment funds Vacations once while the Savings pool still
shows its 7,000 account holdings. A linked ticket purchase can use existing goal funding or additional budget
funding spent immediately; source account choice alone does not decide this.

## Walkthrough

### 1. Start tracking without old history

The user creates accounts and enters each opening balance with a date. The
date defaults to today but can be changed. Lukas calculates later balances
from confirmed transactions and keeps opening-balance provenance inspectable.

**Expected result:** The user can start without importing old activity and can
later compare any account type with a real balance. A discrepancy is shown
first. They can review and match activity, record an explicit adjustment for
any remaining unexplained difference, or leave reconciliation unfinished.
An adjustment appears as a distinct, labeled account entry with its source
and history. It corrects the balance, shows its effect on available-to-assign,
and is not ordinary category spending.

### 2. Bring in this month's activity

The user imports a CSV or Excel statement for one account. They review rows
individually or confirm selected rows in bulk. Likely duplicates are held out
for an individual keep, link/merge or discard decision. Unconfirmed rows stay
in the review list and do not affect reports. The user may leave a row
uncategorized; it then appears under `Uncategorized`.

Rows missing a required value or with an unclear type are flagged. The user
can correct the row before confirming it; its original imported values remain
inspectable. All transaction fields can be corrected during review, while the
row remains associated with the account selected for the file.

If a card purchase has a pending estimate, it counts in spending after import
confirmation. When the posted amount arrives, it replaces the estimate in the
report while the earlier amount and update remain inspectable.

**Expected result:** Only confirmed imported activity affects reports. The
source row and any review or correction decision remain traceable.

If the user later corrects a confirmed transaction, they edit it directly;
the actual date and earlier details/change history remain inspectable. If the
correction affects a closed budget, account and transaction-report history
reflect the correction while the closed budget snapshot is preserved. Where
possible, its budget impact is applied as an adjustment in the current period.

### 3. Understand spending

The user opens the report for the current calendar month and reviews spending
by category and account. They inspect transactions behind a total, assign a
category to an uncategorized row, add tags for context, or split one transaction
across categories. Manually entered transactions require an account, date,
amount and type; transfers also require the other account. Payee, category and
note can be left blank.

Credit-card purchases count on the purchase date by default. A card bill
payment is a transfer. Refunds reduce spending in their category; an unlinked
refund can be categorized by the user. A separately exposed fee defaults to
the related purchase's category while remaining reviewable as statement
detail.

The user can choose per report whether to use actual transaction dates or
assigned periods. Card statement reports can use the close-date or payment-due
date basis; an individual transaction can also receive a manual period
assignment without changing its actual date.

Each card has a statement closing day and payment due day. The due day mainly
serves as a reminder if payment is late; Lukas does not initiate payment. A
statement amount due is used when available and can be corrected by the user;
otherwise it is calculated from recorded account activity through the closing
day. A recorded payment that covers the amount due clears the reminder, and the
user can also mark it as read without changing payment status.

The user gives a bank/card account a base currency, while retaining its
separately denominated balances/wallets. Budget eligibility and pool assignment
stay with the account. A multicurrency pool shows applicable money balances
once, by currency and as a current CLP estimate with rate/date.

For example, a 5 October EUR 100 purchase becomes USD 110 card debt at the
bank's actual conversion (1.10 USD/EUR). A report may initially estimate
CLP 104,000 for the purchase. On 25 October the bank converts this USD 110
at 940 CLP/USD into CLP 103,400 debt. The USD portion is replaced by that CLP
debt, not added to it. Both conversion dates, amounts, rates and sources are
inspectable, and the original EUR purchase/date remain visible. Spending
becomes CLP 103,400 once, a CLP 600 decrease from the earlier estimate. Review
and any affected closed-budget correction follow the existing rules. A later
market rate cannot change the now-fixed CLP 103,400 debt.

A bank that directly converts a JPY 10,000 purchase to CLP 60,000 records that
actual step (6 CLP/JPY) instead. Without a bank conversion, keep the balance in
the spent currency and show CLP only as a labeled reporting estimate. Actual
conversion evidence and report estimates remain distinct; neither conversion
nor a later card payment creates a second expense.

If the statement instead converts a USD 100 balance to CLP 90,000 without
purchase-level amounts, known composition might be USD 60 of Vacations
purchases and USD 40 of opening debt. Lukas suggests CLP 54,000 and CLP 36,000
respectively, labeled as derived allocations, for review/confirmation with
manual changes allowed. The opening-debt part is not new spending. If
Vacations previously had a CLP 53,000 estimate, the confirmed allocation
updates that purchase/goal usage valuation by CLP 1,000 once, without adding
goal funding. Bank-supplied purchase amounts take precedence when available.

Partial conversion suggestions apply only to the known eligible converted
portions and preserve settled portions. Allocations reconcile to the bank
total with any rounding difference visible. Payments remain transfers;
uncertain payment applications require review. Fees/refunds keep their
existing meanings. With incomplete composition, record the actual aggregate
conversion while retaining visibly provisional purchase valuations until
review; do not invent components.

For an equal split of an authoritative CLP 1,000 total, suggest
CLP 334 + 333 + 333 and show the extra peso on its portion. Derived allocation
rounding uses largest fractional remainders, with displayed review order for
ties; valid manual changes must still total CLP 1,000. Ordinary precision is
whole CLP pesos or USD cents, while higher source precision/rates remain
inspectable. Round derived portions after calculation, preserve bank-supplied
purchase amounts, and use the same rule for refunds without inventing another
expense. Confirmed portions do not change when the view is later sorted.

**Expected result:** The user can explain a monthly total and distinguish the
underlying account movement, category, period basis and conversion context.

For a future USD 2,000 hotel component, a selected 900 CLP/USD rate estimates
CLP 1,800,000; an optional 2% user spread makes that forecast CLP 1,836,000.
The component remains a foreign cost estimate within CLP planning, not a
foreign goal target. The user can lock a dated estimate or follow the selected
rate; a card-network rate is an option only where available, with no promised
provider integration.

After a USD 2,000 purchase on a USD-denominated card balance, suppose the
recorded provisional CLP expense is 1,800,000 and its CLP budget assignment is
also 1,800,000. If a variable cost estimate later rises to CLP 1,900,000, show
a CLP 100,000 estimated funding gap. Recorded spending/goal usage stays
CLP 1,800,000, category/goal remaining stays zero, and funding stays
CLP 1,800,000. Current net worth independently shows the USD debt's latest
CLP value. Significant-change alerts use a threshold policy still to define.
A purchase-day lock remains provisional and cannot override the bank amount.

If the bank finally converts that debt to CLP 1,880,000, confirm the evidence
and apply the CLP 80,000 expense difference once. Ordinary goal usage becomes
CLP 1,880,000 and available becomes CLP -80,000 with funding still
CLP 1,800,000; the relevant reservation/correction follows existing rules.
Preserve a closed budget snapshot and apply its correction in the current
period where possible. Eligible held foreign assets follow the separate
automatic budget-funding revaluation rule below; the expense estimate choice
does not change that rule.

**Later-release range example:** When the FX range view is delivered, a
CLP 1,000,000 goal target with USD 500 in its selected expected backing
resources can show CLP 450,000–500,000 of coverage under 900–1,000 CLP/USD
scenarios, or 45–50% coverage of the target, alongside its separate fixed
CLP funded progress. This does not earmark those resources or revalue goal
funding. Future foreign costs can also show ranges; higher FX increases their
CLP cost and the CLP value of held foreign assets. Use the same scenario on
both sides and count each wallet once. Recorded balances/spending and the
floating-estimate funding-gap policy remain unchanged; that warning remains
in the MVP. The range view is later scope. Historical lookback choices such
as six months or a year, calculation method, trend treatment and sources still
require decisions for that later capability; no future bounds or integrations
are promised.

### 4. Review pools and goals

The Savings pool shows 7,000 held in grouped accounts. Vacations separately
shows its 5,000 of budget-assigned funding toward a 10,000 target. Pool holdings
identify expected payment resources and do not add another 7,000 to the goal.
Show linked goal target totals for context.

The user can budget directly toward Vacations and Computer using greedy
priority allocation or paced monthly assignments, for example 1,000 and 500.
They can also budget into a General Savings category used by goal-to-buy
candidates. With 4,000 shared savings, goals costing 3,000 and 2,000 are each
individually affordable, but both together exceed the shared balance. Buying
the 3,000 goal leaves 1,000 in General Savings and records its goal usage once.
Shared goals show cost, affordability and used amount. Vacations, as a
specifically funded goal, shows its assigned funding and funded progress.

Pools still identify expected payment sources and fallback order; actual
transactions identify the accounts used. Their coverage signals remain
separate from affordability against the shared budget category.

For a specifically funded Vacations goal with 5,000 funded and no prior usage,
the user can assign another 1,000 and then categorize a 1,000 ticket purchase
to Vacations, even when the payment account is outside its expected pool. Both
steps can be offered in one flow. The result is 6,000 funded, 1,000 used and
5,000 available. Recording only the purchase leaves 4,000 available from the
original 5,000 funding. Category spending and goal usage count the expense once.

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

For example, correcting a CLP 200,000 portion from Goal A to Goal B reduces
Goal A's usage by CLP 200,000 and increases Goal B's usage by the same amount.
If A and B have distinct specifically funded categories, restore CLP 200,000
of availability to A's category and apply the spending to B's category once.
The parent purchase remains one expense with the same amount and actual date.

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

### 5. Give available money a job

The user starts a zero-based budget for the current calendar month with
eligible money not already assigned at the period start, plus each income
receipt once. The user
chooses budget eligibility per account; a paycheck is not counted again just
because it appears in the updated account balance. They assign money to spending
categories and goals; pools continue to group accounts and are not budget
destinations. At period close, the user can choose per category or goal
whether unspent money carries forward or returns to the unassigned amount.
Carried money remains assigned next period and is excluded from new unassigned
money, even though it remains in the account balance. Budget directly to
specific goals or to the General Savings category used by shared-fund goals.
Pools are expected payment sources, not a second funding assignment.

A card purchase uses its category budget when made and virtually reserves only
genuinely cash-covered budgeted money for the later bill. Paying the bill
remains a transfer without a second assignment for already reserved money.

For example, with CLP 600 genuinely available in a category, a CLP 1,000
card purchase records CLP 1,000 spending and debt, leaves category availability
at CLP -400, reserves CLP 600 for the bill and shows CLP 400 uncovered debt.
Before any payment, a later explicit CLP 400 assignment/reassignment backed
by eligible money brings category availability to zero and the reservation to CLP 1,000;
spending and debt remain CLP 1,000. Merely entering an unsupported CLP 400
assignment does not cover that debt or increase the cash-covered reservation.

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

For the original CLP 1,000 debt/CLP 600 reservation state, a CLP 500
payment leaves debt CLP 500 and reservation CLP 100, with CLP 400 still
uncovered. Alternatively, paying CLP 800 leaves debt CLP 200 and zero
reservation: CLP 200 was paid without coverage and CLP 200 remains unpaid.
These portions explain the original CLP 400 gap, not an additional shortfall.
Later genuine funding of CLP 400 covers CLP 200 already paid and reserves
only CLP 200 still owed. Payment itself changes neither spending nor category
overspending. (AC-5.15)

Starting with a CLP 1,000 purchase funded with CLP 600 in its original
category, CLP -400 original-category availability, CLP 600 reserved and
CLP 400 uncovered debt, a linked
CLP 300 refund leaves debt CLP 700, category availability CLP -100, reservation
CLP 600 and uncovered debt CLP 100. No reservation is released yet.
Starting instead from that same original state, a CLP 500 refund leaves debt
CLP 500, category availability CLP 100, reservation CLP 500 and no uncovered
debt. Both cases reduce spending once without income or new ready-to-assign.
(AC-5.16)

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

Existing credit-card debt reduces the money available to assign at the start
of the budget; the credit limit never counts as available money. Later card
purchases consume their category budget and reserve its genuinely covered
portion for the bill, without deducting the purchase a second time.

The user chooses whether this budget follows actual transaction dates or
assigned reporting periods. This supplies defaults, but each purchase must
resolve to exactly one budget month. Mixed monthly settings that would include
a purchase twice or omit it require resolution. Its category impact and bill
reservation follow the same resolved month, while its real date stays visible
and each report retains its independent period choice.

For example, eligible income received in October can fund November once; the
same money is not also available for another October assignment. A 1,000 card
purchase dated 28 October can be assigned to November's budget. Even if
October's actual-date basis and November's assigned-period basis both nominate
it, resolving it to November uses its category funding and bill reservation
there once. An actual-date spending report still shows the October purchase.
If the card is paid on 10 November, November's actual cashflow shows that
payment as a transfer with a bank outflow and debt reduction, not another
expense. The payment date does not determine the purchase's budget month.

For an eligible JPY 5,000 asset balance initially valued and assigned at
CLP 30,000, a current valuation of CLP 32,000 automatically adds CLP 2,000 to
ready-to-assign. Explain it as an FX valuation adjustment with source/rate/date,
not a new income receipt. The existing CLP 30,000 assignment/funded goal value
stays fixed unless the user assigns the extra CLP 2,000. A fall instead to
CLP 28,000 exposes a CLP 2,000 shortfall/over-assigned warning, without a lock
or silent reassignment. Account estimates also update on tracking-only assets,
but those changes do not fund the budget. Actual JPY stays JPY unless the bank
actually converts it; a known incoming conversion uses its actual amount.
Keep source-review/final-replacement rules, count transfers/receipts once, and
preserve closed snapshots while applying current valuation effects once.
Automatic asset valuation is MVP behavior; an automatic-versus-reviewed user
choice is later. This does not automatically revalue USD expense estimates;
foreign-debt coverage follows AC-5.17.

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

The month remains open until the user explicitly closes its budget. The user
can still add or edit transactions dated in that month after the calendar
month ends. At close, they review carryover choices. If a closed month later
needs correction, the correction takes a more deliberate path; where possible
the transaction is corrected directly with its actual date and audit history
preserved, the closed budget snapshot stays intact, and its budget impact is
applied as a current-month adjustment.

**Expected result:** A partially assigned budget can be saved and shows the
amount still waiting for a job. A budget that assigns more than is available
can be saved and shows the shortfall as `over-assigned by [amount]`.

Before forecasting, consider a separate budget-boundary example: with
CLP 100,000 funding, CLP 80,000 assigned and CLP 20,000 ready to assign, the
user transfers CLP 30,000 into a tracking-only account. Funding becomes
CLP 70,000; assignments stay CLP 80,000, so the budget shows CLP 10,000
over-assigned. Explain the funding reduction separately; no spending or goal
usage is created. Transfers in the opposite direction add funding once, while
transfers within budgetable accounts or within tracking-only accounts have
no budget-boundary funding effect. Independent FX differences and exposed
fees retain their existing treatment, counted once. Actual dates and closed
snapshots retain their existing
correction rules. Payments of already-recognized debt retain their covered or
paid-uncovered treatment under AC-5.19 without another ordinary boundary
deduction for either portion. Unrelated asset transfers keep the boundary rule. (AC-5.11)

The user can also change an account's budget eligibility after reviewing a
preview. With CLP 100,000 funding, CLP 80,000 assigned and CLP 20,000 ready
to assign, enabling a tracking-only account holding eligible CLP 50,000 adds
that current balance once from confirmation onward. Funding becomes
CLP 150,000 and ready to assign becomes CLP 70,000. Disabling an eligible
account removes its current contribution once and can expose over-assignment.
Keep assignments/funded goals fixed; no actual balance, bank data, real date,
independent categorization, report history or closed budget snapshot changes.
No transfer, income/spending or category/goal usage is created. Original
receipts and prior valuations are not counted again; existing foreign-asset
valuation rules remain in force. Debt-bearing changes retain recognized debt
and coverage under AC-5.19. (AC-5.12)

For a separate initial-funding example, budgetable opening balances of
CLP 100,000 in one bank, CLP -20,000 in another and CLP 5,000 of actual positive
card credit provide CLP 85,000 before assignments. Negative bank/cash/savings
balances reduce funding; actual card credit adds funding. Unused credit or
overdraft limits provide nothing. Preserve the opening card-debt rule and
count later purchases/reservations once without another debt deduction or
automatically treating positive card credit as fresh income. Credit
consumption follows AC-5.18; purchase coverage follows AC-5.14. (AC-5.13)

An eligibility change also preserves already-recognized debt: with bank
CLP 100,000 and CLP 95,000 already protected for USD 100 card debt,
ready-to-assign stays CLP 5,000 when the card becomes tracking-only. Its later
covered payment consumes that protection once; it does not also remove another
CLP 95,000 as an ordinary budget-boundary outflow. Positive assets still leave
funding prospectively; enabling a debt-bearing tracking account recognizes
only previously unrepresented debt once, preserving previous coverage and
showing genuine shortfalls rather than inventing funded reserves. Actual
balances, sources, dates and closed snapshots remain intact. The same principle
applies to recognized negative bank balances. Existing FX, payment, refund and
positive-credit rules still apply. Already-recognized uncovered debt also
keeps its paid-uncovered treatment without another boundary deduction: debt
CLP 1,000/reserved CLP 600/gap CLP 400, changed to tracking-only and then paid
CLP 800, leaves CLP 200 paid-uncovered plus CLP 200 still-unpaid uncovered.
Those two amounts are the original CLP 400 gap, not a new additional
shortfall. Unrelated asset transfers still follow AC-5.11. (AC-5.19)

### 6. Forecast the coming months

The user chooses a historical lookback and a future horizon. For this
projection, they choose whether recurring rules or history-based estimates
take precedence when both describe the same expense. Each simple recurring
occurrence has an editable estimate, can be skipped or rescheduled, and can be
linked to a confirmed or imported actual transaction. Lukas can suggest a
likely match, but the user confirms it. The linked actual replaces the estimate
for that occurrence in the projection; both remain distinguishable in history.

The projection shows estimated income, spending, net cashflow and progress
toward dated goals. Goal savings reduce projected money left after expenses
but appear separately from spending. Estimates do not modify actual history.
If a scheduled goal purchase may overlap a recurring or history-based estimate,
the user marks whether they describe the same cost or different costs. A same-
cost pair counts once, with both sources visible; different costs both count.
Until the user marks them as the same cost, keep both separate and count both.
Lukas does not infer an overlap silently.

A scheduled goal purchase reduces projected pool balances and the balances
carried into later periods. The projection draws first from the goal's preferred
pool, then from subsequent pools in spending-priority order, and shows each
pool's contribution. Reserve targets are warning thresholds, so fallback may
draw below them. Drawdowns stop at zero; any remaining purchase cost is shown
as unfunded. Known dated obligations reduce a projected pool balance when
they occur, not upfront. For example, CLP 100,000 remains available before a
known CLP 40,000 payment's due date; when the payment occurs, projected balance
falls to CLP 60,000 and a later purchase uses that balance. Count a projected
outflow only once. This does not change actual accounts or budget assignments.
This simple fallback is part of the first release. User-defined
account or pool restrictions and events that unlock resources are later scope.

**Expected result:** The user can explain how the projection used history,
recurring estimates and goal savings, and can distinguish each estimate from
confirmed actual activity.

## First-release behavior checklist

- Start accounts from dated opening balances and calculate balances from
  confirmed activity.
- Complete the required account fields with explicit opening amount confirmation;
  budget eligibility does not limit categorization or reporting.
- Add transactions manually and import one-account CSV/Excel files with mapping
  and format previews; valid rows can be confirmed while bad rows stay in review.
- Review imported rows individually or in bulk; keep unconfirmed and suspected
  duplicate rows out of reports until resolved.
- View current-month spending by category and account, including pending-to-
  posted updates, refunds, fees, uncategorized rows and category splits.
- Preserve actual transaction dates while allowing per-report period choices
  and per-card statement-period rules.
- Show multicurrency values with the chosen CLP conversion context.
- Show current net worth across tracked assets and liabilities; defer all exports.
- Group whole accounts into pools as expected payment sources.
- Treat specifically funded goals as budget categories with targets; use
  greedy or paced budget allocation strategies for their assignments. Let
  `goal to buy` goals share a budget category, with purchases consuming that
  category once. Show saved/used/available values, manual goal links and
  saved/target progress.
- Create simple recurring expense rules and projections without turning
  estimates into actual transactions.
- Project scheduled goal purchases against the preferred pool and subsequent
  pools by spending priority, showing pool contributions and any unfunded cost.
- Create a zero-based budget by assigning available money to categories and
  goals, with a per-category or per-goal choice for unspent-money carryover.
- Reconcile account balances by showing differences and letting the user
  choose a resolution process.

### Review current net worth and spending comparisons

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

## Questions for the walkthrough review

1. Is there a step here that should move out of the first release, even though
   the broader capability remains part of the product?
2. Does the order reflect how you expect a new user to reach their first useful
   monthly spending report?
3. Which expected result is unclear or missing?

Other unresolved product decisions remain listed in `03-spec-v1.md`; this
walkthrough does not settle them.
