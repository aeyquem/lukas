# Lukas product specification v1

Status: First product-spec draft for review. It records the scope and behavior
discovered so far; open decisions are called out explicitly. This document
does not select implementation technology or architecture.

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

## 1. Product purpose

Lukas helps a person in Chile understand where their money went, give available
money a job through zero-based budgeting, and see how their current finances
may develop. It brings together accounts, transactions, purpose-based pools,
goals, recurring expenses and projections while making financial figures
reviewable and explainable.

The first user is one person managing their own finances. The initial
experience is a web application. Self-hosting is the accepted initial
deployment direction, with a possible future SaaS path.

## 2. Product principles

- **Chile first:** use CLP as the first reporting currency and Chilean money,
  date and financial conventions as defaults.
- **Keep original financial facts visible:** preserve transaction dates,
  currencies, source information, and pending or final amounts when present.
- **Keep history auditable:** retain the source of manual and imported
  activity, preserve corrections and user decisions, and record account
  differences through explicit, traceable resolution rather than silent
  balance rewriting.
- **Keep money precise:** monetary amounts and exchange rates use decimal-safe
  representation and calculations; floating-point rounding must not change
  financial values. Reports explain conversion observations rather than hide
  how a CLP value was reached. The storage representation remains undecided.
- **Review before trusting imports:** unreviewed imported rows do not affect
  reports. Corrections and user decisions remain inspectable.
- **Separate actuals from estimates:** projections and recurring forecasts do
  not silently become actual transactions.
- **Explain amounts:** reports distinguish actual, pending, projected,
  adjusted and reconciled values when that affects interpretation.
- **Organize, do not transact:** Lukas reads and organizes financial
  information. It does not move money or initiate payments or transfers.

## 3. First-release scope

Zero-based budgeting is a core product requirement and belongs in the first
release, using calendar-month periods to match the default monthly spending
view. The user assigns available money to spending categories and goals;
pools only group accounts and are not budget assignment destinations.
Available-to-assign starts with eligible money not already assigned at the
period start, plus each income receipt once. An income receipt is not counted
again because it appears in the updated account balance. At period close, the
user chooses per category or goal
whether unspent money carries forward or returns to the unassigned amount.
Carried amounts remain assigned next period and are excluded from new
unassigned money, even while the funds remain in account balances. Which
accounts are budgetable is chosen per account. A partially assigned budget can
be saved and shows money waiting for a job. Over-assignment is allowed and
shows the amount as `over-assigned by [amount]`. Existing credit-card debt
reduces money available to assign; credit
limits are not money. New card purchases use their category budget once and
virtually reserve only its genuinely cash-covered portion for the later bill
(AC-5.14), so the same purchase or card balance movement is not deducted a second time. Goal funding comes from
budget assignments, directly for a specifically funded goal or through a shared
category for goal-to-buy purchases. Pools identify expected payment sources.
A period remains open until the user explicitly closes it. While it is open,
the user can add or edit transactions dated in it, even after its calendar
month ends. Closing includes reviewing carryover choices. A closed period can
still be corrected through a more deliberate correction flow. Correct the
transaction directly while retaining its actual date and audit history;
preserve the closed budget snapshot and, where possible, apply the budget
impact as an adjustment in the current period.

MVP budget assignments, category/goal funding, goal targets and monthly
contribution plans use CLP. Foreign-currency budget and goal-target choices
are later scope. A planned cost component may nevertheless retain an amount
such as USD 2,000 and show its estimated CLP cost; that component is not a
USD-denominated goal target.

### 3.1 Accounts and balances

The user can track bank, cash, credit-card, savings and investment accounts.
Accounts retain their real currencies. New accounts can start from an opening
balance dated today by default, with an option to choose another date. The
calculated balance follows confirmed activity after that opening date; users
do not need to import their complete prior history.

An opening balance is recorded as dated, traceable opening-adjustment activity
with its source/provenance, rather than as an unexplained mutable account
starting value. The user can inspect and correct its history.

The user can compare a calculated balance with a real balance and reconcile any
account type. Reconciliation first shows the difference; the user chooses how
to resolve it each time. Reconciliation must not silently rewrite activity to
hide a mismatch.

Investment accounts include a basic account value and its variation from the
prior period. Holdings and investment transactions such as deposits or
interest are later scope.

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

Before confirming an eligibility change, show its budget funding effect.
From confirmation onward, changing tracking-only to budgetable adds the
current eligible positive asset balance once; the reverse removes it once. Apply this
prospectively rather than replaying past receipts or changing closed budgets.
It changes no actual balance, bank source data, real transaction date,
categorization or report history. No transfer, income, expense, category use
or goal usage is created. Keep assignments and funded goals fixed and show
any resulting over-assignment. Retain existing foreign-asset valuation rules
without counting prior valuation effects again. Debt and its existing coverage
survive eligibility changes as specified in AC-5.19. (AC-5.12)

Initial funding uses actual signed budgetable balances. Negative bank, cash
or savings balances reduce funding; an actual positive card credit adds it.
Unused card credit and overdraft limits add nothing. For example, CLP 100,000
in one bank, CLP -20,000 in another and an actual CLP 5,000 card credit give
CLP 85,000 before assignments. This is initial available money, not automatic
new income. Preserve the existing opening card-debt rule and count later
transactions, category spending and bill reservations once; do not subtract
debt twice. Positive-credit consumption follows AC-5.18; returning owned
money retains transfer treatment. Purchase coverage follows AC-5.14. (AC-5.13)

For a debt-bearing account, eligibility controls positive asset funding but
cannot erase debt already recognized by the budget. Making a card or negative
bank account tracking-only retains its recognized debt and its existing
protection, reservations or uncovered status until settled. Do not release
ready-to-assign, reverse past spending or deduct the same liability again.
Enabling budgetability recognizes only previously unrepresented debt once,
under existing opening-debt rules, preserving prior coverage and exposing
shortfalls rather than inventing genuinely funded reserves. A later payment
of already-recognized debt keeps its once-only treatment even if the receiving
account is now tracking-only. Covered portions consume retained protection;
uncovered portions retain AC-5.15's paid-uncovered gap treatment. Neither
portion also receives an ordinary boundary-outflow funding deduction.
Unrelated asset transfers and positive-credit portions retain their existing
rules. Changes remain prospective, previewed
and confirmable with warnings, without changing actual balances, bank data,
real dates or closed snapshots. (AC-5.19)

### 3.2 Transactions and monthly spending

Owned money transfers across the budget boundary have a funding effect,
separate from their financial classification. Transfers between two budgetable
accounts or two tracking-only accounts have no budget-boundary funding effect.
This does not suppress independent FX differences or exposed fees under their
existing rules. For example, a cross-currency transfer between budgetable
accounts whose held-money CLP valuation was 100,000 and whose actual converted
destination amount is CLP 95,000 retains the CLP 5,000 difference once under
the existing FX rules; it is not erased because no budget boundary was crossed.
Separately exposed fees retain their category treatment without double counting.
Tracking-only to budgetable adds funding once; budgetable to tracking-only
removes funding once. Payments of already-recognized debt under AC-5.19
retain their existing covered or paid-uncovered treatment without another
boundary-outflow deduction for either portion. The movement remains a transfer, not income or spending.
Keep assignments and funded goals fixed; a funding reduction can leave the
budget visibly over-assigned. Do not automatically use a category, assign
money to a goal or record goal usage because money crossed the boundary.
Preserve real dates and closed snapshots under existing correction rules.
Initial signed balances follow AC-5.13; positive-credit consumption follows
AC-5.18 and returns retain transfer treatment. Linked refund coverage follows
AC-5.16 and purchase coverage AC-5.14. Eligibility
changes follow AC-5.12, with retained debt/protection handled by AC-5.19.
(AC-5.11)

The user can enter activity manually or import CSV and Excel statements. The
first spending report defaults to the current calendar month and supports
breakdowns by category and account.

The user can record income, expenses, transfers, fees, refunds and explicit
adjustments. A transfer between accounts owned by the user is one logical
movement and does not count as income or spending. Cash is an account; an ATM
withdrawal is a transfer into cash, while cash purchases are separate expenses.
Cross-currency transfers retain both account currencies and the conversion
observations that explain the values on each side.

Every transaction requires an account, date, amount and type: income, spending
or transfer. A transfer also requires the other account. Payee, category and
note are optional; an uncategorized transaction remains visible as
uncategorized.

Transactions may have one category or be split across categories. Categories
describe what an expense was for. Built-in categories are editable; users can
add categories. Expenses can remain uncategorized and appear in an
uncategorized group until the user assigns a category. Transactions can have
multiple user-created tags for context or why, such as `transport` tagged
`grandma-visit`.

Credit-card purchases count as spending when made. Paying a card bill is a
transfer, not another expense. A refund reduces spending in its category. If
the original purchase is not obvious, the user can categorize the refund and
it reduces that category. A separately exposed fee defaults to the related
purchase's category while remaining reviewable as statement detail.

In the zero-based budget, a card purchase uses its category budget when made
and virtually reserves only genuinely cash-covered budgeted money for the later
bill. The bill payment is a transfer and does not need a second budget
assignment for already reserved money.

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

Each budget lets the user choose a period basis: actual transaction date or
assigned reporting period. This supplies the default budget attribution;
every purchase affects exactly one budget month. If monthly settings would
include it twice or omit it, require resolution to one month. Category impact
and a card purchase's virtual bill reservation follow that resolved month.
The actual transaction date remains visible and unchanged, and report period
settings remain independent. A later basis change must not silently duplicate
or drop existing purchase impacts; changes affecting closed budgets preserve
their snapshots and use the agreed current-period correction process where
possible.

Income already received can be assigned to a later monthly budget, becoming
available there once. The same receipt or money assigned for that later month
is not counted again or left simultaneously available for another assignment.
An actual-date cashflow view shows a card payment in its actual payment month
as a payment/transfer, including the bank outflow and card-debt reduction. It
creates no second expense or budget assignment, and does not change the
purchase's resolved budget month.

When a card transaction is pending, its estimated amount counts in spending
after the import is confirmed. When the posted amount arrives, it replaces the
pending estimate in the spending value while the pending and posted context
remain inspectable.

Manual transactions also retain their source history. Editing reviewed
activity changes the transaction directly while keeping its prior details and
subsequent changes inspectable and preserving the actual transaction date. A
correction updates account and transaction-report history. If it affects a
closed budget, preserve that budget's close history and apply the budget impact
as an adjustment in the current period where possible. Corrections must meet
the same interpreted-value requirements as new confirmed activity.

### 3.3 Import and review

Each CSV or Excel file represents one account in the first release. The user
can review individual rows or confirm selected rows in bulk. Unconfirmed rows
remain available in a review list and do not affect reports.

Rows missing required values or with an unclear transaction type are flagged
for review. The user can correct them before confirmation; unconfirmed rows do
not affect balances or reports, and original imported values remain
inspectable. No transaction fields are locked during review; each row remains
associated with the account selected for its file.

Likely duplicate rows are held for an individual decision outside bulk
confirmation. The user can keep a row, link or merge it with existing activity,
or discard it. The same source and decision history should remain reviewable.

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

### 3.4 Dates, periods and card statements

The actual transaction date remains unchanged when the user assigns a
transaction to another reporting period. Each report can choose whether it
uses the actual date or the assigned period. For card statements, a report can
also use the statement close-date period or payment-due-date period. Users can
set per-card statement-period rules and manually assign a transaction to a
different period.

For a card with a statement close near the 25th and payment due on the 1st of
the next month, the report's chosen period rule determines which month shows
the purchase. The actual purchase date and actual payment date remain
available independently. Each card has a recurring statement closing day and
payment due day. The due day primarily supports a reminder if the bill is not
paid on time; Lukas does not initiate payments. Recorded payments that cover
the statement amount due clear the reminder. Use the statement's amount due
when available, allowing the user to correct it; otherwise calculate the due
amount from recorded account activity through the closing day. The user can
also mark the reminder as read without changing payment status.

### 3.5 Multicurrency reporting

An account has a base currency and may hold several separately denominated
balances/wallets, including on bank and card accounts. Its base currency is
independent from CLP reporting currency and does not convert balances by
itself. Budget eligibility and pool grouping remain per-account choices.

Retain the original purchase currency/amount separately from the currency and
amount affecting the account's funds/debt. Keep the bank's actual conversion
chain with input/output amounts, currencies, source, rate when available or
derivable, and actual conversion dates. A JPY/EUR purchase may convert
immediately to CLP, remain in the spent currency, or become USD card debt that
converts to CLP at billing-cycle end. Record each real conversion without
creating another purchase or keeping the replaced debt as additional debt.
Keep conversion dates distinct from the original purchase date.

For non-CLP spending, use the final attributable CLP amount shown by the
statement when available; otherwise use a transaction-date estimate with its
source and rate. A report estimate alone does not establish that the bank
converted funds/debt. Pending/provisional account amounts remain labeled as
such. Confirmed posted/final evidence replaces the previous estimate or
applies its difference once, preserving review history and the established
closed-budget/current-period correction rules. Keep all earlier observations
inspectable. Reconciliation compares balances in their actual currencies;
current rates cannot overwrite fixed converted debt or actual conversion
amounts. No fee or intermediate conversion is inferred without evidence.

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

For cross-currency transfers, keep both account-side amounts in their actual
currencies and retain the conversion observations used to relate them.

Multicurrency pools show their original-currency amounts and a current CLP
estimate using the latest available rate, with the rate and date shown.

#### Amount precision and split rounding

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

Handling source totals exceeding the normal allocation precision still needs
a product rule; do not truncate or replace the supplied amount.

### 3.5.1 Current net worth and report comparisons

The MVP includes a current net-worth snapshot: account assets less liabilities
across tracked accounts, including accounts excluded from budgeting. Use actual
dated balances, independently of assigned reporting periods. Show each balance's
date, original currencies, and a current CLP estimate with conversion source,
rate and date; never imply that missing conversion data is zero. Pool holdings
and budget assignments are views of those assets and are not added again.
Each separately denominated balance contributes once to account and current
net-worth totals. A base-currency summary is not another asset or liability.
Outstanding foreign balances receive a current CLP display estimate; an
actual conversion replaces the converted portion with the new denomination.
Historical combined net-worth trends require a later valuation rule.

Spending views support date range, account, category and optional tag filters,
with underlying transactions inspectable. Category filtering of a split expense
includes matching portions once, excludes unmatched portions from totals, and
retains the parent relationship for inspection. Period comparisons show both
exact date ranges, absolute change and percentage change; when the prior value
is zero, percentage change is unavailable. A fully customizable report builder
and all exports are deferred beyond the MVP.

### 3.5.2 Foreign cost estimates and funding gaps

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

Eligible foreign asset budget funding follows the separate automatic
revaluation rule in section 3.5.4; the cost-estimate setting does not control
that asset policy.

### 3.5.3 FX ranges: later release

The following accepted display semantics describe the later FX scenario/range
capability. The agreed funding-gap warning in section 3.5.2 remains MVP scope.

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

### 3.5.4 Automatic revaluation of eligible foreign assets

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

### 3.5.5 Foreign-debt valuation and budget coverage

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

### 3.6 Purpose-based pools

Pools group whole money-holding accounts in the first release. Credit-card
transactions may be associated with a pool; card liability and credit limits
are excluded from pool holdings. Show original currencies and current CLP
estimates with rate/date context.

Pools identify expected payment sources and show actual holdings separately
from budget-designated money. They do not separately fund goals. Show total
linked goal targets as informational context; this summary is provisional.
Keep calculated resource availability and user-set spendable amounts distinct.
When the manual amount is used for planning, show its comparison separately.
These are payment-source coverage signals; goal-to-buy affordability is based
on the shared budget category described below.
In projections, known dated obligations reduce the simulated pool balance on
their occurrence date, not upfront. Count each projected outflow once; linked
goal targets are informational and are not obligations.

The prior pool-level goal reservation system is superseded. Spending-order
fallback and reserve warning thresholds are included in the first-release
projection. User-defined scenario restrictions and unlock events, partial
account allocations, and percentage/fixed-amount/remainder allocation rules
remain later scope.

### 3.7 Goals

A specifically funded goal behaves as a budget category with a target
attached. It receives ordinary budget assignments, including initial funding
from eligible opening money ready to assign. Purchases are categorized to
that goal category regardless of the account used. Selecting its category and
recording its goal usage do not create separate expenses. A goal has a
target amount and optional target date. Show required monthly savings for a
dated goal; an undated paced goal can use a manually chosen monthly amount.

Greedy and paced describe budget allocation strategies. Greedy funds goals in
priority order and passes remaining allocatable money to the next goal. Paced
assigns a monthly amount to each goal. Per goal, retain the choice of automatic
allocation or suggestions that the user confirms.

A goal-to-buy uses a shared budget category, such as General Savings. The
budget funds that category; the goal does not individually reserve its balance.
Compare each goal cost with the shared category balance, and warn if combined
goal costs exceed it. Buying one reduces the category balance once and records
goal usage. Do not count category spending and goal usage as two expenses.
The shared category balance is not separate funding for every candidate goal.
Shared goals show cost, affordability and used amount. Do not display the
shared category balance as an individually saved amount or funded-progress
bar for each candidate; those funding measures belong to specifically funded
goals.

For specifically funded goals, saved is net funding after explicit funding
reassignments and other approved reductions, including amounts already used.
Available is saved minus used; progress is saved divided by target. For
example, 5,000 saved including 1,000 used leaves 4,000 available and gives 50%
progress toward a 10,000 target. Carried assignments count once.

Each goal also identifies its expected payment pool. Pool holdings are distinct
from budget funding and are not added to a goal's saved amount. Transactions
record the accounts actually used. Manual goal linking uses a dedicated
relationship on each purchase portion, separate from tags; an unsplit purchase
has one portion. Automatic confirmed goal suggestions remain later scope.
For a specifically funded goal purchase, use the ordinary assignment and
spending steps: optionally assign additional money to its category, then
record the expense against it. The product can offer both steps together while
keeping the assignment and expense separately inspectable. Choose existing
goal funding or additional budget funding spent immediately. The former increases used and
reduces available; the latter increases funded and used equally, preserving
prior available funding. Source accounts/pools alone do not create funding.
An additional assignment uses ready-to-assign money or explicit reassignment;
over-assignment remains allowed with the agreed shortfall warning. Paying the
card bill remains a transfer, not a second expense or goal use.

Ordinary goal usage is the default. The user can explicitly choose an
exceptional transaction that reduces saved instead of increasing used. Keep
existing used visible, recalculate availability, and retain the choice in
history.

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

### 3.8 Cashflow projections

The first-release projection deducts scheduled goal purchases from projected
pool balances, and later periods carry the reduced balances forward. If a
purchase exceeds its preferred pool's resources, cover the shortfall using
subsequent pools in the user's spending order and show the contribution from
each. Known dated obligations reduce the affected projected pool balance when
they occur, before later purchases draw from that pool. For example, CLP
100,000 in a pool remains CLP 100,000 before a known CLP 40,000 payment is
due, then becomes CLP 60,000 when it occurs; a later goal purchase sees that
CLP 60,000. Do not subtract an outflow again if it is already represented in
the projection. This projection does not change actual accounts or monthly
budget assignments. Reserve targets describe desired retained resources and act as
warning thresholds. Fallback may draw below a target; show the resulting
shortfall against it.

If all eligible pools are exhausted, stop projected drawdowns at zero and
show the remaining cost as an unfunded shortfall.

**Later scope:** The user can mark a pool and/or account as restricted and
define events that unlock it for planning. Apply restrictions before allocating
projected spending; exclude resources until their user-defined conditions are
met, and keep total holdings distinct from scenario-eligible resources.
External processes for obtaining access to funds are outside scope. Pool and
account restrictions apply together: an account is eligible only when both
its own conditions and its pool's conditions are satisfied. Unlocking either
one does not override the other's restriction. The user manually activates
unlocking events within each projection and chooses their effective dates.
These events are scenario assumptions and do not change actual accounts or
monthly budgets.

The user can choose the historical lookback period and number of future months
for each projection. Projections use recent income and spending patterns and
show estimated income, spending, net cashflow and progress toward dated goals.
The assumptions and projected status remain visible; estimates do not mutate
actual history.

Goal savings reduce the projected money left after expenses, but remain
separate from spending. For each projection, the user chooses whether recurring
rules or history-based estimates take precedence when both estimate the same
expense. This source choice is separate from linking a confirmed transaction to
a forecast occurrence. When a scheduled goal purchase and a recurring or
history-based estimate may describe the same expected cost, let the user mark
them as the same cost or as different costs. Count a same-cost pair once and
count a different-cost pair twice. Until the user marks them as the same cost,
keep both separate and count both. Do not infer the relationship silently;
retain both sources and the user's decision in the projection assumptions.
When the user marks a pair as the same cost but the estimates differ, which
amount drives the projected cost remains open.

### 3.9 Recurring expenses

Simple recurring expenses are included. A rule can repeat every N days, on a
selected day of each month, at month-end, or on an ordinal weekday such as the
last Friday. Rules can have an end date; individual forecast occurrences can
be skipped or rescheduled. Each occurrence has an estimate whose amount the
user can adjust independently.

Recurring occurrences are forecast estimates. Lukas may suggest a match when a
confirmed or imported actual transaction could correspond to an occurrence;
the user confirms the link. Once linked, the actual replaces that occurrence's
estimate in the projection, while both remain distinguishable in history.
Multi-transaction
recurring event templates, such as a birthday trip with separate gas, toll and
gift expenses, are later scope.

## 4. Later scope

- A CLI for people and agents to query and add data.
- Open Finance connections; the first release uses manual entry and CSV/Excel.
- Native shared-expense tracking for one-time outings and ongoing groups,
  including optional Splitwise read/write as a backend. Shared expenses can be
  paid by the user or others, split equally, by custom amounts, percentages or
  item-by-item, and settled with a choice per settlement about linking an
  account transaction. Repayments are not counted again as income or spending.
- Investment holdings and activity such as deposits and interest.
- Automatic transaction-to-goal suggestions, subject to user confirmation.
- Partial account-to-pool allocations and advanced allocation rules.
- Multi-transaction recurring event templates.

Lukas never moves money, even when it later reads or writes shared-expense data
with Splitwise.

## 5. Candidate use cases

### UC-1: Understand monthly spending

As a person managing my finances, I want to see spending by category and
account for the current calendar month, inspect the underlying transactions,
and distinguish actual, pending and estimated amounts so I can understand
where my money went.

### UC-2: Add or import activity

As a user, I can enter transactions manually or import a statement file, review
rows individually or in bulk, resolve likely duplicates separately, and keep
unconfirmed or unresolved rows out of reports.

### UC-3: Start and reconcile an account

As a user, I can start an account from a dated opening balance without its full
history, calculate its balance from confirmed later activity, compare it with
a real balance, review and match activity, create an explicit adjustment for
an unexplained difference, or leave reconciliation unfinished. Any adjustment
is a distinct, labeled account entry: it corrects the account balance, shows
its effect on available-to-assign, and is not treated as ordinary category
spending. Its source and history remain visible. This flow is available for
any account type.

### UC-4: Organize accounts by purpose

As a user, I can group whole money-holding accounts into pools, see original
currencies and a dated CLP estimate, distinguish pool holdings from budget
assignments to goals, and optionally set a separate spendable amount for
planning. Pools indicate expected payment sources and do not reserve or fund
goals.

### UC-5: Plan and track a goal

As a user, I can create a specifically funded goal that receives budget
assignments or a goal-to-buy that compares its cost with a shared budget
category. I can choose greedy or paced budget allocation, see saved,
used/available values and monthly savings needs for specifically funded goals,
link a purchase to its goal, and identify its expected payment pool without
confusing goal progress with account transfers.

### UC-6: Project cashflow

As a user, I can choose a historical lookback and future horizon, see projected
income, spending, net cashflow and dated-goal progress, and understand the
assumptions behind each estimate.

### UC-7: Forecast recurring expenses

As a user, I can define a simple repeat rule, adjust an occurrence's estimate,
skip or reschedule an occurrence, and link the actual transaction when it
happens.

### UC-8: Make a zero-based budget

As a user, I can assign money available in accounts plus income received in
the period to spending categories and goals, see unassigned or over-assigned
amounts, and choose per category or goal whether unspent money carries forward
or returns to the unassigned amount. Pools group accounts and are not budget
destinations. I can fund specific goals directly or assign money to a shared
savings category used by goal-to-buy candidates; pools identify expected
payment sources.

All exports, including transaction and report exports, are later scope.

## 6. Open product decisions

These questions remain open and should not be treated as settled by this draft:

- What does a CLI query/add workflow require when it arrives, including safe
  writes for people and agents?
- How should Splitwise synchronization, collaborator access, partial
  settlements and netting across groups work when shared expenses are planned?
- Which additional reports and goal/projection comparisons are needed after
  the specified first-release flows? All exports are deferred.

## 7. Product-level success signals to define

The primary outcome is that a user can understand where money went. Before
implementation, the team should define measurable success signals such as
whether users can set up an account, review an import and explain their monthly
spending and projected remaining money. Specific targets are not yet set.
