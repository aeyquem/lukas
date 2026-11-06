# MVP acceptance criteria v1

Status: Working product checklist derived from `03-spec-v1.md` and
`04-mvp-walkthrough.md`. These criteria describe observable product behavior;
they do not select technical implementation. Items that depend on unresolved
decisions are labeled open rather than treated as acceptance requirements.

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

## 1. Account setup and balances

### AC-1.1: Start with an opening balance

Given a user creates an account, when they enter an opening balance, then the
date defaults to today, can be changed, and the opening balance's source and
history remain inspectable.

### AC-1.2: Calculate an account balance

Given an opening balance and confirmed activity after its date, when the user
views the account, then the calculated balance reflects that confirmed
activity. Unconfirmed imported rows do not change it.

### AC-1.3: Reconcile a balance

Given a calculated account balance and a real balance, when the user compares
them, then Lukas shows the difference first and lets the user review and match
activity, create an explicit adjustment for an unexplained difference, or
leave reconciliation unfinished. An adjustment is a distinct, labeled
account entry that corrects the balance, shows its effect on
available-to-assign, and is not counted as ordinary category spending. Its
source and history remain inspectable. Lukas does not silently change prior
activity to hide the difference. This is available for any account type.

### AC-1.4: View an investment account value

Given an investment account, when the user views its balance history, then
Lukas shows its basic account value and variation from the prior period.
Holdings and investment transactions remain later scope.

### AC-1.5: Complete account setup

Account creation requires a nonblank name, account type, base currency,
opening balance and date, and budget eligibility. Offer CLP, today and zero as
defaults, but ask the user to explicitly confirm the opening amount and its
currency; distinguish card debt from money held. Bank, savings, and investment
accounts track one currency per account; card and cash accounts may have several
separately denominated balances/wallets. Cash in another currency may also use
a separate cash account. Retain dated openings without requiring unused
wallets. The base currency does not force conversion of a wallet balance.
Bank, cash, savings and credit-card accounts default to budgetable; investments default to tracking only. Eligibility is editable per
account and controls budgeting only, without limiting categorization or
reporting. Institution, account reference, notes and pool assignment are
optional. Opening money enters ready-to-assign once under the existing
eligibility and debt rules.

## 2. Manual entry, imports and review

### AC-2.1: Import a statement

Given a CSV or Excel statement for one account, when the user imports it, then
they can review individual rows or confirm selected rows in bulk. Unconfirmed
rows remain available for review and do not affect reports or calculated
balances.

### AC-2.2: Resolve a suspected duplicate

Given an imported row likely matching existing activity, when bulk confirmation
is performed, then that row is held for an individual decision. The user can
keep it, link or merge it with existing activity, or discard it; the source and
decision remain inspectable.

### AC-2.3: Leave an expense uncategorized

Given a manual or imported expense without a category, when the user saves or
confirms it, then it appears under `Uncategorized` until a category is
assigned.

### AC-2.4: Update a pending card amount

Given a confirmed pending card transaction with an estimated amount, when its
posted amount arrives, then the posted amount replaces the estimate in
spending while the pending and posted context remains inspectable.

### AC-2.5: Categorize an unlinked refund

Given a refund without an obvious original purchase, when the user assigns a
category, then the refund reduces spending in that category and remains
inspectable as its own activity.

### AC-2.6: Show an exposed fee

Given a statement exposes a fee separately from a purchase, when Lukas presents
it, then it defaults to the purchase's category and remains reviewable as
statement detail.

### AC-2.7: Correct a confirmed transaction

Given a confirmed transaction, when the user corrects it, then they edit the
transaction directly, its actual date remains intact, and they can inspect its
previous details and subsequent changes. Account balances and transaction
reports reflect the correction. If it affects a closed budget, that budget's
close history remains preserved and the budget impact is applied as a
current-period adjustment where possible.

### AC-2.8: Enter a transaction

Given the user enters a transaction manually, then account, date, amount and
type (income, spending or transfer) are required. A transfer also requires the
other account. Payee, category and note are optional; an uncategorized
transaction remains visible as uncategorized.

### AC-2.9: Correct an incomplete imported row before confirmation

Given an imported row is missing a required value or has an unclear
transaction type, when the user reviews it, then Lukas flags the issue and
allows correction of any transaction field before confirmation. The original
imported values remain inspectable, the row remains associated with the
account selected for its file, and it does not affect balances or reports
until confirmed.

### AC-2.10: Preview mappings and resolve malformed imports

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

## 3. Spending, transactions and periods

### AC-3.1: Review current-month spending

Given confirmed activity, when the user opens the default spending report,
then it shows the current calendar month with breakdowns by category and
account, and the user can inspect the underlying transactions.

### AC-3.2: Categorize and tag activity

Given an expense, when the user categorizes it, then they can use an editable
built-in or user-created category, split it across categories, or leave it
uncategorized. They can also add multiple user-created tags; tags remain
separate from categories and goal links. A manual goal link identifies each
linked split portion's goal under AC-5.22. It is optional for ordinary/shared
categories; choosing a specifically funded goal category requires its matching
association.

### AC-3.3: Count a credit-card purchase once

Given a credit-card purchase and a later bill payment, when spending and
cashflow are reported using actual dates, then the purchase counts as
spending on its purchase date and the bill payment appears in its actual
payment month as a payment/transfer, with the bank outflow and debt reduction.
The payment is not a second expense. In the budget, the purchase affects its
resolved budget month once; its payment month does not automatically change
that attribution. Reports may independently use their agreed period choices.

### AC-3.4: Choose a report period basis

Given a transaction with an actual date and an assigned reporting period, when
the user selects a report basis, then the report can use either the actual
date or assigned period without rewriting the actual date. For card statements,
the report can use the close-date or payment-due-date period. Individual
transactions can also receive a manual period assignment.

### AC-3.7: Set card statement and due dates

Given a credit-card account, when the user configures it, then they can set
its recurring statement closing day and payment due day. The due day supports
a reminder if payment is late; Lukas does not initiate payments. Recorded
payments covering the statement amount due clear the reminder. Use the
statement amount due when available and allow the user to correct it; otherwise
calculate it from recorded card activity through the closing day. The user can
also mark the reminder as read without changing payment status.

### AC-3.5: Report a non-CLP amount

Given a non-CLP transaction, when a final CLP statement amount is available,
then spending reports use it. Otherwise they use a transaction-date estimate
and show its source and rate. The original currency and amount remain
available.

### AC-3.9: Track account currency balances and card conversion chains

Given a bank, savings, or investment account, then it has one tracked currency
wallet. Given a card or cash account with a base currency and multiple currency
balances/wallets, preserve each balance's denomination independently of the
base and reporting currencies. Cash in a different currency may also use a
separate cash account. Budget eligibility and pool assignment remain per-account
choices. Account, applicable pool and net-worth CLP totals include each relevant
currency balance once; the base-currency summary is not an additional balance,
and card debt is not pool money.

Given a EUR 100 purchase that the card issuer converts to USD 110 debt and
later to CLP 103,400 debt at billing-cycle end, then retain the original EUR purchase
and each actual conversion's input/output amounts, currencies, source, date
and available/derivable rate. On the later conversion date, replace the
converted USD 110 portion with CLP 103,400; do not keep both as debt or create
another expense. The original purchase date stays unchanged. Direct JPY/EUR
to CLP conversions and balances retained in the spent currency follow the
same evidence rule, without invented conversion steps or fees.

Given an earlier CLP 104,000 spending estimate for that purchase, when the
final attributable CLP 103,400 statement amount is confirmed, then replace
the estimate once (a CLP 600 decrease), retaining the prior observation and
applying existing review and closed-budget/current-period correction rules.
A reporting estimate is not itself an actual account conversion. Provisional
account amounts remain labeled; reconciliation uses actual currency balances.
Later current-rate updates do not change the fixed CLP 103,400 debt. If the
bank has not converted a foreign balance, keep that denomination and use
current CLP conversion only for the appropriate display valuation.

### AC-3.10: Review allocations from an aggregate bank conversion

Given exact bank-supplied purchase CLP amounts, then use those amounts before
internal allocations. Given only an aggregate conversion, keep its amounts
authoritative for the account and label purchase/category/goal allocations as
derived. With known composition, suggest proportional allocation across the
eligible outstanding portions actually converted; require review/confirmation
and allow manual changes. Bank-identified portions take precedence. Totals
reconcile to the bank amount and any rounding discrepancy remains visible.

For example, USD 100 converted to CLP 90,000 comprises USD 60 of tracked
Vacations purchases and USD 40 of opening debt. Suggest derived allocations
of CLP 54,000 and CLP 36,000. Only the first contributes to tracked purchase
spending and goal usage; opening debt does not create new spending. A previous
CLP 53,000 purchase estimate becomes CLP 54,000 once, with no automatic goal
funding. Retain source/decision history and real dates; apply established
closed-budget/current-period correction rules where necessary.

For a separate partial conversion of USD 50 from that same outstanding mix
into CLP 45,000, suggest allocating converted portions of USD 30/CLP 27,000
to Vacations and USD 20/CLP 18,000 to opening debt. Leave the other USD 50
outstanding and preserve any previously settled portions. Payments are
transfers counted once; any uncertain derived payment application requires
review. Exposed fees and refunds retain their expense/credit meanings and
category links rather than turning the entire conversion into new spending.

Given incomplete or uncertain composition, then the actual aggregate currency
conversion can still be recorded while purchase valuations remain visibly
provisional until reviewed. Do not invent purchases, fees or settled portions.

### AC-3.11: Distinguish foreign cost estimates from recorded spending

Given a future foreign cost component, then retain its foreign amount and show
a CLP estimate using a selected current rate and optional user spread. A
selected card-network rate is usable where available; this does not promise
provider integration or a guaranteed bank charge. Let the user lock a dated
estimate or follow the selected rate. Before purchase, it affects forecasts
and estimated funding gaps rather than actual spending.

Given USD 2,000 of actual card debt and an initially recorded provisional CLP
expense of 1,800,000 against CLP 1,800,000 of funding, when its chosen variable
cost estimate rises to CLP 1,900,000, then show a CLP 100,000 estimated funding
gap. Do not automatically alter recorded spending/goal usage, funding,
remaining amounts or reservation: usage stays CLP 1,800,000 and remaining
stays zero. Current net worth values the real USD debt independently at the
latest available rate. A locked purchase-day estimate remains provisional.

When actual confirmed bank conversion establishes CLP 1,880,000 for that
purchase, then apply the CLP 80,000 difference once under the existing source
hierarchy and review/correction rules, including closed-budget handling.
Ordinary goal usage becomes CLP 1,880,000, available becomes CLP -80,000, and
funding stays CLP 1,800,000. Related reservation handling follows the existing
funding rules. Actual bank evidence overrides locked or variable estimates;
reviewed derived aggregate allocations remain labeled as derived.

Significant changes govern alerts about the estimate; the threshold policy
remains open. Eligible foreign assets follow the separate automatic budget
funding rule in AC-5.10; cost choices do not control that asset policy.

### AC-3.12: Round derived portions without changing authoritative totals

Use currency-appropriate precision for ordinary entry/display: whole CLP pesos
and USD cents. Preserve higher precision supplied by a source and its rates
for inspection. Calculate derived portions before rounding; do not overwrite
bank-supplied purchase amounts or silently alter an authoritative source total.

Given CLP 1,000 split equally three ways, then suggest CLP 334 + 333 + 333.
Distribute residual smallest units to portions with the largest fractional
remainders, breaking ties by displayed portion order used during allocation
review. Show the extra peso on its portion. Allow valid manual changes while
requiring that the portions sum exactly to the authoritative CLP 1,000.
A USD 1 equal split analogously gives USD 0.34 + 0.33 + 0.33. Later view sorting
must not silently change confirmed allocations.

Apply the same rule to refunds with signs preserved, and to derived aggregate
conversion allocations in their relevant currencies. Show remainder
adjustments rather than introducing another expense or account effect.
Unusual source totals with precision beyond ordinary allocation units remain
an edge case for a further product rule; retain the exact supplied amount.

### AC-3.6: Record a cross-currency transfer

Given a transfer between owned accounts in different currencies, when it is
recorded, then both account-side amounts retain their original currencies and
the conversion observations are inspectable. The transfer is not income or
spending.

### AC-3.8: Inspect net worth and filtered comparisons

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

## 4. Pools and goals

### AC-4.1: Group whole accounts into pools

Given money-holding accounts, when the user creates purpose-based pools, then
each account can be assigned in full to one pool in the first release.
Partial account allocations and percentage/fixed-amount splitting are later
scope.

### AC-4.2: Show multicurrency pool values

Given a pool containing accounts in multiple currencies, when the user views
it, then they can see each original-currency amount and a current CLP estimate
with the rate and date.

### AC-4.3: Keep card liability out of pool money

Given credit-card transactions associated with a pool, when the pool total or
available amount is shown, then the card's liability and available credit are
not counted as pool money. Card transactions remain available for review and
reporting.

### AC-4.4: View pool resources

Given a pool, show its grouped account holdings, calculated resource
availability, user-set spendable amount and informational total of linked goal
targets. Show budget-designated money separately. A pool supplies expected
payment-source context; its holdings are not additional goal funding.
In projections, a known dated obligation reduces the projected pool balance
when it occurs, not before. Do not count goal targets as obligations, and do
not subtract an obligation again when it is already represented by the same
projected outflow.

### AC-4.5: Plan a goal to buy from shared savings

Given multiple goal-to-buy goals using a shared General Savings category,
compare each cost against that category's available balance without reserving
individual amounts. Show when the combined costs exceed the shared balance.
Buying a goal reduces the category balance once and records usage for that
goal, without counting a second expense. Pool coverage is a separate signal;
shared-category funds must not count as independent funding for each goal.
Show each shared goal's cost, affordability and used amount; individually saved
amounts and funded-progress bars belong to specifically funded goals.

### AC-4.6: Fund a goal through the budget

Given eligible money ready to assign, when the user assigns an amount to a
goal in the budget, then the goal records that funding once and the amount
remaining ready to assign decreases accordingly. This applies to the initial
assignment of existing money as well as later income. Account and pool
balances do not change merely because money receives a job.

### AC-4.7: Show saved, used and available goal values

Given a specifically funded goal with a target, net saved/funded amount and
goal-related usage,
when the user views progress, then `saved` includes amounts already used,
`available` is saved minus used for a specifically funded goal, and first-release progress
is saved divided by target. For example, a 10,000 goal with 5,000 saved,
including 1,000 used, shows 4,000 available and 50% progress.

### AC-4.8: Choose how to fund a goal purchase

A specifically funded goal behaves as a budget category with a target.
Categorize the purchase to its goal category regardless of payment account.
Offer ordinary assignment followed by expense recording, optionally together
in one flow; keep both steps visible and count the expense once across category
and goal views. With 5,000 funded and no prior usage, an additional 1,000
assignment followed by a 1,000 expense gives 6,000 funded, 1,000 used and 5,000
available. Without the assignment, it gives 5,000 funded, 1,000 used and 4,000
available.

Given a purchase linked to a specifically funded goal, let the user choose to
use existing funding or assign additional budget money and spend it
immediately. Using existing funding increases used and reduces available.
Additional funding increases funded and used equally; prior available funding
remains intact. Budget money must be accounted for through ready-to-assign or
explicit reassignment; over-assignment can be saved with the agreed warning.
Account/pool choice alone does not determine funding. A transfer or later card
bill payment is not another contribution or expense.

Example: 5,000 funded plus 500 newly assigned and transferred into savings,
plus 1,000 newly assigned and spent on tickets gives 6,500 funded, 1,000 used,
and 5,500 available. Count the transfer-related assignment once.

### AC-4.9: Distinguish goal funding from expected payment source

Given a budget-funded goal linked to a pool, when the user views it, then
budget-assigned funding and the pool's account holdings are distinct figures.
The pool is the expected source of payment; its holdings are not another
contribution to goal funding. There is no separate pool-versus-budget funding
mode to select.

### AC-4.10: Set a goal amount and savings pace

Given a goal, when the user defines it, then they can set a target amount with
or without a target date. For a dated goal, Lukas shows the amount needed to
save each calendar month. For an undated paced goal, the user can set a monthly
amount.

### AC-4.11: Mark an exceptional reduction to saved

Given a transaction linked to a goal, when the user reviews the link, then
ordinary usage is the default. The user can explicitly mark the transaction to
reduce cumulative saved instead of increasing used. Existing used remains
visible, availability is recalculated, and the exceptional choice remains
inspectable.

### AC-4.12: Allocate to goals through the budget

Given eligible money ready to assign, greedy allocation funds goals in chosen
priority order and passes remaining allocatable money to the next goal. Paced
allocation assigns a monthly amount per goal, such as 1,000 to Vacations and
500 to Computer. Allocation happens in the budget; pools indicate expected
payment sources. Automatic allocation versus confirmed suggestions remains a
per-goal choice.

## 5. Zero-based budgeting

### AC-5.1: Start a monthly budget

Given the current calendar month, when the user opens the budget, then the
available-to-assign amount starts with eligible money not already assigned at
the period start plus each income receipt once. A received paycheck is not
counted again merely because it appears in the updated account balance. At
initial setup, eligible existing money enters ready-to-assign once and can be
assigned immediately to a category or goal.

### AC-5.2: Choose budgetable accounts

Given any account, when the user configures their budget, then they can choose
per account whether it participates in budgeting. Initial actual signed
budgetable balances follow AC-5.13. Existing credit-card debt reduces
available-to-assign, and credit limits never count as money.

### AC-5.3: Assign money to jobs

Given available-to-assign money, when the user makes a budget, then assignments
go to spending categories and goals. Pools group accounts and are not budget
destinations.

### AC-5.4: Save an incomplete or over-assigned budget

Given a budget with money still unassigned, when the user saves it, then Lukas
shows the amount waiting for a job. Given assignments exceed available money
by an amount, when the user saves the budget, then Lukas allows it and shows
`over-assigned by [amount]`.

### AC-5.5: Carry money between periods

Given unspent money in a category or goal at period close, when the user
chooses carryover, then it remains assigned next period and is excluded from
new unassigned money even while the funds remain in account balances. If the
user returns it to unassigned, it is available to assign again.

### AC-5.6: Budget a card purchase once

Given a category budget and a credit-card purchase, when the purchase is
recorded, then it reduces the category's available budget by the full purchase
and reserves only genuinely cash-covered budgeted money for the later card
bill (AC-5.14). When the bill is paid, the transfer uses available reservation
without a second assignment for that covered amount. Each budget can use
actual transaction dates or assigned reporting periods to
supply defaults. The purchase's category impact and bill reservation follow
one resolved budget month, without changing the actual transaction date.
Mixed monthly settings cannot count it twice or omit it; resolve conflicts as
specified in AC-5.8.

Existing credit-card debt reduces the amount available to assign at the start
of the budget. The account's credit limit is excluded. A later purchase uses
its category budget and bill reservation once; its card-balance change does
not reduce available-to-assign a second time.

### AC-5.7: Close and correct a budget period

Given a calendar-month budget, when the month ends, then it remains open until
the user explicitly closes it. While open, the user can add or edit
transactions dated in that period. Closing requires reviewing carryover
choices. After closing, corrections remain possible through a more deliberate
flow. Correct the transaction directly with its actual date preserved; keep
the closed budget snapshot and apply the budget impact in the current period
where possible.

### AC-5.8: Attribute purchases and received funding once across months

Given a purchase dated 28 October and assigned to November, when October's
actual-date basis and November's assigned-period basis would both include it,
then require resolution to one budget month. Apply its category impact and
card bill reservation once in that resolved month. If the inverse settings
would omit it, require resolution rather than silently dropping its budget
impact. The real date remains visible and report period choices stay
independent.

Given eligible income received in October, when the user assigns that money
to November's budget, then it funds November once and is not simultaneously
available for another October assignment. The receipt is not counted again
when November begins or when the updated account balance is read.

Given a later date-basis or attribution change, then existing purchase impacts
must not silently duplicate or disappear. Preserve closed snapshots and follow
the agreed current-period correction process where possible. Paying a card
bill in November appears in November's actual cashflow as a payment/transfer,
not a second expense, and does not automatically change its purchase's budget
month.

### AC-5.9: Plan budgets and goal targets in CLP

MVP budget assignments, category/goal funding, goal targets and monthly
contribution plans are expressed in CLP. Foreign budget/goal-target currency
choices are later scope. A foreign-currency cost component, such as a planned
USD 2,000 hotel, can have a locked or variable CLP estimate under AC-3.11
without making the budget or goal target USD-denominated.

### AC-5.10: Automatically revalue eligible foreign assets for budget funding

Given eligible foreign money held (assets), when its daily mid-market CLP value
changes, then update budget funding automatically via an explainable valuation
adjustment with rate/source/effective date, separately from ordinary
income/spending. On weekends and holidays, use the latest published daily rate
and show its effective date. Frankfurter v2 is the selected provider. If the
provider is unreachable, use the latest stored observation, show its effective
date and mark it stale. If no observation is stored, mark the foreign valuation
unavailable; do not treat it as zero or return a partial CLP budget-funding
total.
For JPY 5,000 initially valued and fully assigned at CLP 30,000, a new current
value of CLP 32,000 adds CLP 2,000 ready-to-assign once. Existing assignments/funded goal amounts remain
CLP 30,000 unless the user assigns/reassigns; targets retain their set values.
If the value instead falls to CLP 28,000, show the CLP 2,000 shortfall as an
over-assigned warning without a lock or forced assignment changes.

Tracking-only accounts also receive current account/report valuations but no
budget funding effect. Foreign balances retain their currency unless actually
converted; a known bank-converted incoming amount remains actual in the
resulting currency. Preserve source review, pending/final replacement, and
once-only receipt/owned-transfer rules. Isolate the FX difference instead of
counting previously recorded transaction effects again. Preserve closed budget
snapshots and apply current effects under established correction rules, once.

Do not apply this asset policy to floating foreign expense estimates: those
still change only estimated funding gaps until actual conversion/confirmed
allocation changes the original expense once. Current foreign-debt valuation
for net worth remains independent; warning-only debt coverage follows AC-5.17.
Automatic asset revaluation is
MVP behavior; the user choice of automatic versus reviewed valuation is later
scope and does not introduce wallet-level settings.

### AC-5.11: Transfer money across the budget boundary

Given an owned money transfer, transfers between two budgetable accounts or
two tracking-only accounts have no budget-boundary funding effect. Independent
FX differences and exposed fees retain their existing rules and are counted
once. For example, a cross-currency transfer between budgetable accounts with
CLP 100,000 of prior held-money valuation and an actual CLP 95,000 destination
retains the CLP 5,000 difference once under the existing FX rules. Exposed fees
keep their category treatment; do not duplicate a difference already accounted
for. Tracking-only to budgetable increases funding once; budgetable to
tracking-only reduces it
once. Show an explainable budget funding effect separately from financial
income/spending; the movement stays a transfer in reports. Do not silently
change assignments or funded goals, create a transfer category, or record
goal usage. Preserve actual dates and closed snapshots under existing
correction rules. Payments of already-recognized debt under AC-5.19 retain
their existing covered or paid-uncovered treatment; neither portion receives
another ordinary boundary-outflow deduction. Unrelated asset transfers still
follow this boundary rule.

For example, CLP 100,000 of budget funding with CLP 80,000 assigned leaves
CLP 20,000 ready to assign. Transferring CLP 30,000 to a tracking-only account
leaves CLP 70,000 of funding and the unchanged CLP 80,000 assigned: show
`over-assigned by CLP 10,000`. The same amount remains owned and tracked; no
expense or automatic reduction of goal funding is created.

Initial signed balances follow AC-5.13; subsequent card-credit consumption
and refunds are not settled by this transfer rule. Purchase coverage follows
AC-5.14. Account eligibility changes follow AC-5.12.

### AC-5.12: Preview and confirm a prospective eligibility change

Given an account with a current eligible positive asset balance, show its funding effect
before confirming a change of budget eligibility. From confirmation onward,
tracking-only to budgetable adds that current balance once; the reverse
removes it once. Keep assignments and funded goals fixed, allowing a visible
over-assignment warning. This adjustment creates no physical movement,
income, expense, category use or goal usage, and changes no actual balance,
original bank data, real transaction date, independent categorization/report
history or closed budget snapshot. Do not replay original receipts or prior
valuation adjustments. Existing foreign-asset valuation rules remain in force;
recognized debt and its coverage survive changes as specified in AC-5.19.

For example, CLP 100,000 funding and CLP 80,000 assigned leave CLP 20,000
ready to assign. Enabling a tracking-only account holding eligible CLP 50,000
shows a CLP 50,000 increase in the preview; after confirmation, funding is
CLP 150,000 and ready to assign is CLP 70,000. Conversely, starting from
CLP 100,000 funding and CLP 80,000 assigned, disabling an account contributing
CLP 30,000 leaves CLP 70,000 funding and CLP 10,000 over-assigned, without
changing the CLP 80,000 assignments. Neither change moves money or reclassifies
historical transactions as income/spending.

### AC-5.13: Count actual signed budgetable opening balances

Given initial budget funding, actual negative balances in budgetable bank,
cash or savings accounts reduce funding, and actual positive credit-card
credits add funding. A positive card credit is money owed to the user, not
unused borrowing capacity. Neither a credit limit nor an overdraft limit
provides funding. Account eligibility continues to govern participation.

For example, budgetable opening balances of CLP 100,000 in Bank A,
CLP -20,000 in Bank B and CLP 5,000 of actual card credit provide CLP 85,000
before assignments. Keep the existing opening card-debt deduction and do not
deduct that debt again. Positive card credit does not automatically become
new income; subsequent receipts, transactions, card purchases and bill
reservations affect the budget once under their existing rules. Fixed
assignments/funded goals remain fixed, with over-assignment visible when needed.

Positive-credit consumption follows AC-5.18; returning owned money retains
transfer treatment. Purchase coverage follows AC-5.14. Debt-bearing eligibility changes follow
AC-5.19 rather than being decided by this opening-balance example.

### AC-5.14: Record card overspending and cover it explicitly

Given CLP 600 of genuinely cash-covered category availability, no positive
card credit and a valid CLP 1,000 actual card purchase, when the user records it, then spending and
debt both increase by the full CLP 1,000, category availability becomes
CLP -400, the bill reservation is CLP 600 and uncovered debt is CLP 400.
If no category money is available, reserve zero and show the full purchase
as uncovered debt. Do not sweep ready-to-assign or other categories.

Given the CLP 400 shortfall before any payment, when the user explicitly
assigns or reassigns CLP 400 backed by eligible money to cover it, then category availability
becomes zero and the bill reservation becomes CLP 1,000. Spending and debt
remain CLP 1,000. An unsupported assignment can be saved with an
over-assignment warning but must not increase the cash-covered reservation
or hide uncovered debt. Count the covered amount once, without reducing
ready-to-assign again for the card-balance change.

Budget, category and bill-coverage shortfalls warn; they do not prevent
recording an actual purchase or payment allowed by the bank, even when a
payment exceeds its reservation. Lukas does not initiate or authorize banking
transactions. Required fields, import confirmation and ambiguity/duplicate
review still apply; a budget warning does not make unresolved data valid.
Credit/overdraft limits supply no funding. Preserve opening-debt, real-date,
resolved-budget-month, closed-snapshot, FX and fixed-funding policies.
Partial-payment coverage follows AC-5.15 and refund coverage AC-5.16;
ordinary goal-linked refunds follow AC-5.20; exceptional/excess cases remain
open. Positive-credit consumption follows AC-5.18.

### AC-5.15: Apply partial card payments without new spending

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

Given a CLP 1,000 purchase with CLP 600 reserved, CLP 400 uncovered and
category availability CLP -400, with no opening debt or positive card credit,
when an actual same-currency CLP 500 bank-to-card payment is recorded between
budgetable accounts, then debt becomes CLP 500, reservation CLP 100 and
uncovered debt CLP 400. Spending remains CLP 1,000 and category availability
remains CLP -400; the CLP 500 is not freed for another assignment.

Starting instead from the same original state, a CLP 800 payment leaves
CLP 200 debt and zero reservation. Show CLP 200 paid without prior coverage
and CLP 200 still-unpaid uncovered debt. Together they explain the original
CLP 400 gap; do not add another CLP 200 or CLP 400 shortfall. The actual
payment remains a transfer in its payment month and is not blocked.

If the user then explicitly provides CLP 400 of genuine funding for that
original gap, CLP 200 covers its already-paid portion and only CLP 200 is
reserved for outstanding debt. Category availability becomes zero; spending
stays CLP 1,000 and debt stays CLP 200. Record no second expense and do not
require another assignment for amounts already protected by opening-debt
funding. Payment application to specific bank purchases/statements requires
the appropriate evidence; the budget rule does not invent that evidence.

### AC-5.16: Restore refund coverage without duplicate funding

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

For a separate fully funded CLP 1,000 purchase, starting with original-category
availability zero, debt/reservation CLP 1,000 and no other card activity,
a CLP 300 refund before payment leaves debt/reservation CLP 700 and category
availability CLP 300. Alternatively, if the CLP 1,000 was already paid in full,
the refund leaves actual positive card credit CLP 300, reservation zero and
category availability CLP 300. Net spending is CLP 700 in either case. Do not
release another CLP 300 from an already-consumed reservation or also add it to
ready-to-assign. The original purchase and refund stay separately inspectable;
use the actual refund date and the established closed-period correction flow.
Ordinary goal usage reversals follow AC-5.20; exceptional/excess goal refunds
and unsupported bank payment matching are not settled here. Positive-credit
consumption follows AC-5.18.

### AC-5.17: Revalue foreign-debt coverage without automatic budget changes

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

### AC-5.18: Spend actual positive card credit before borrowing

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

If the same card starts with CLP 300 actual credit but only CLP 200 genuinely
funded category availability for that CLP 500 purchase, record the full CLP 500
expense and CLP 200 new debt, leaving category availability CLP -300. Reserve
zero: CLP 100 of existing credit was used without category coverage and CLP 200
is uncovered new debt, together the original CLP 300 gap. Do not count either
portion again or create funded reservation from an unsupported assignment.

If the credit arose from a confirmed refund, preserve that refund's category
restoration and history. The next purchase uses its own category once; do not
restore the refund category again or add the consumed credit to ready-to-assign.
A payment's portion exceeding actual debt remains positive card credit rather
than an uncovered debt payment; record its asset transfer once. Unused card
borrowing limits never count as real positive credit. Debt-bearing eligibility
changes follow AC-5.19.

### AC-5.19: Preserve recognized debt across eligibility changes

Preview and confirm changes as in AC-5.12. Positive eligible assets cease or
begin funding prospectively, but making a debt-bearing card or negative-bank
account tracking-only does not erase already-recognized liabilities or undo
their budget effects. Preserve existing opening-debt protection, purchase
reservations and uncovered amounts until settled under their existing rules.
Do not release ready-to-assign, reverse spending or deduct protected debt
again merely because eligibility changed.

When enabling a debt-bearing tracking account, recognize only the portion not
already represented in the budget once under existing opening-debt rules.
Preserve previous coverage; an insufficient backing amount can expose a
shortfall/over-assignment but cannot create genuinely funded coverage. Keep
assignments/funded goals fixed and allow confirmation with warnings. Actual
balances, source facts, dates, independent reports and closed snapshots remain
intact. Foreign debt continues to use AC-5.17; payments, refunds and positive
credits retain AC-5.15, AC-5.16 and AC-5.18.

For example, a bank holds CLP 100,000 and USD 100 of existing card debt already
has CLP 95,000 recognized/protected, leaving CLP 5,000 ready to assign. Making
the card tracking-only leaves that CLP 5,000 unchanged and retains the debt's
CLP 95,000 protection. A later CLP 95,000 payment uses that protection once:
no new expense, assignment or second boundary-outflow funding deduction is
created for the protected amount. Foreign-debt rate changes still follow the
warning and actual-conversion rules rather than this eligibility setting.

Conversely, where CLP 95,000 of debt genuinely has no previous budget effect
or coverage, enabling participation recognizes it once. With CLP 100,000 of
unassigned backing available this leaves CLP 5,000 ready to assign; existing
assignments may instead leave an over-assignment warning. Do not perform this
deduction again if the debt was already represented by an opening-debt effect
or a purchase/category impact and reservation.

Payments of already-recognized uncovered debt also keep AC-5.15's once-only
treatment when the receiving account is tracking-only. Paying such a portion
does not repair its funding gap: show the paid-uncovered amount instead of
adding another boundary-outflow funding deduction. For example, CLP 1,000 debt
with CLP 600 reserved and CLP 400 uncovered becomes tracking-only. A CLP 800
payment consumes CLP 600 coverage and pays CLP 200 without prior coverage;
it leaves CLP 200 unpaid debt and zero reservation. The original CLP 400 gap
is CLP 200 paid-uncovered plus CLP 200 still-unpaid uncovered, not that gap
plus another CLP 200 boundary deduction. Category availability stays CLP -400
and spending stays CLP 1,000 until explicitly funded under AC-5.15.

The same principle applies to a recognized negative bank balance: a budget
starting with CLP 100,000 in Bank A and CLP -20,000 in Bank B has CLP 80,000
before assignments. Making Bank B tracking-only does not free another
CLP 20,000. A covered payment clearing that balance consumes its retained
coverage once, without a second ordinary boundary-outflow deduction.
Previously recognized uncovered portions likewise retain their existing gap
treatment without another boundary deduction. Unrelated asset transfers and
payment portions creating positive credit follow their existing boundary and
asset rules; do not extend the recognized-debt exception to them.

### AC-5.20: Reverse ordinary linked goal usage on refund

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

In both examples, the category availability increase is the same refund effect
already represented under AC-5.16, not an additional restoration from the goal
link. Preserve the original purchase's recorded usage and show its linked
refund reversal, rather than erase or rewrite the purchase. Actual banking
refunds can still be recorded when an exceptional/excess goal effect remains
unresolved; do not silently choose its goal funding effect.

### AC-5.21: Reassign goal funding without moving actual money

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
This is the warning policy for explicit funding reassignments only. Corrected
purchase-portion goal links follow AC-5.22; exceptional saved-reduction
transactions/refunds remain open questions. (AC-5.21)

For a goal with saved/funded CLP 5,000, used CLP 1,000 and available
CLP 4,000, explicitly moving CLP 500 to another budget category/goal leaves
funded CLP 4,500, used CLP 1,000 and available CLP 3,500. Moving CLP 4,500
instead from the same initial state leaves funded CLP 500, used CLP 1,000 and
available CLP -500. Warn about that single CLP 500 coverage shortfall; the
receiving CLP 4,500 assignment does not prove that all of it has backing.

### AC-5.22: Link split purchase portions to their respective goals

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

Given a confirmed CLP 200,000 purchase portion linked to Goal A, when the user
corrects it to Goal B, then preserve the original link and record the correction,
reduce A's usage by CLP 200,000 and increase B's usage by CLP 200,000. If A and
B use different specifically funded categories, correct the category too and
restore A's category availability while applying B's category spending once.
Keep the purchase amount, account effect and actual date unchanged; do not add
funding or count another expense. If the correction affects a closed budget,
preserve its snapshot and apply the budget impact in the current period where
possible under AC-2.7 and AC-5.7.

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

If an ordinary/shared-category portion has no goal link, include it in
category/account totals without inventing goal usage. Each portion's amount is counted once; the sum of linked
usage need not equal the whole purchase when other portions are unlinked.
Keep original source amounts and the inspectable parent/portion relationship.
A Vacations-category portion cannot lack its Vacations association or link to
Photography. Linking to Photography requires its matching goal category
instead. Category selection and the matching goal effect count the portion
once; this does not introduce automatic goal-link suggestions.
This criterion does not settle refund distribution across portions or
exceptional saved-reduction actions.

## 6. Projections and recurring expenses

### AC-6.1: Create a cashflow projection

Given account and transaction history, when the user creates a projection,
then they choose a historical lookback and future horizon. The projection shows
estimated income, spending, net cashflow and progress toward dated goals;
estimates remain distinguishable from actuals.

### AC-6.2: Show goal savings in a projection

Given monthly goal savings in a projection, when the user reviews money left
after expenses, then goal savings reduce that remaining amount but appear
separately from spending.

### AC-6.3: Create a simple recurring expense estimate

Given a simple recurring expense rule, when future occurrences are projected,
then supported schedules include every N days, a selected day of each month,
month-end and an ordinal weekday. Each occurrence has an independently
adjustable estimate and can be skipped or rescheduled; the rule can have an end
date.

### AC-6.4: Link an actual to a forecast occurrence

Given a forecast occurrence, when a confirmed or imported actual transaction
could match it, then Lukas can suggest the match but the user must confirm it.
Once linked, the actual replaces the occurrence's estimate in the projection,
while both remain distinguishable in history. If a recurring rule and
history-based estimate overlap, the user chooses per projection which source
takes precedence. Separately, overlap between a scheduled goal purchase and an
estimate follows AC-6.6: the user decides whether they describe the same cost.

### AC-6.5: Project goal purchases across pools

Given a scheduled goal purchase in a projection, when its preferred pool does
not cover the full cost, then the projection draws from subsequent pools in
spending-priority order and shows each pool's contribution. Reserve targets
are warning thresholds, so a drawdown may pass below them. Drawdowns stop at
zero, and any remaining cost is shown as unfunded. Projected pool balances
carry into subsequent periods without changing actual accounts or budget
assignments. Apply known dated obligations to projected balances on their due
dates before allocating a purchase that occurs later; each outflow affects a
projected pool balance once.

For example, a pool holding CLP 100,000 remains at CLP 100,000 before a known
CLP 40,000 payment's due date. When that payment occurs in the projection,
the pool falls to CLP 60,000, and a later goal purchase draws from that
projected balance. Do not reduce actual account holdings or budget assignments.

### AC-6.6: Resolve overlap between a goal purchase and an estimate

Given a scheduled goal purchase and a recurring or history-based estimate that
may describe the same expected cost, when the user reviews the overlap, then
Lukas lets the user mark them as the same cost or different costs. If marked
the same, count the expected cost once and retain both sources and the user's
decision in the projection assumptions. If marked different, count both costs.
Until the user marks them as the same cost, keep both separate and count both.
Do not infer an overlap or silently remove either source. These choices affect
the projection only; they do not create or change actual transactions or
monthly budget assignments.

For example, a scheduled CLP 500,000 refrigerator goal purchase in November
and a CLP 500,000 November estimate for appliances can be marked as the same
purchase, producing one CLP 500,000 projected cost, or as separate costs,
producing CLP 1,000,000 of projected costs. Keep the goal purchase and estimate
inspectable in either case. This example uses equal amounts; how to choose the
projected amount when sources marked as the same cost have different estimates
remains open.

## 7. Later scope: user-defined resource restrictions

Users can restrict pools and/or accounts and define events that unlock them for
planning. Exclude restricted resources until their conditions are met; show
total holdings separately from eligible resources. External processes for
accessing funds are outside scope. If an account and its pool both have
restrictions, both sets of conditions must be satisfied before the account
becomes eligible; unlocking one does not override the other. The user manually
activates events and chooses effective dates within each projection. Resources
become eligible from those dates once all applicable conditions are satisfied.
Scenario events do not change actual accounts or monthly budgets. These
user-defined restrictions and unlock events are later scope; first-release
projections use the simple spending-priority fallback in AC-6.5.

Shared-expense tracking and optional Splitwise read/write, CLI, Open Finance,
investment activity, automatic goal suggestions, partial account-to-pool
allocations and multi-transaction recurring-event templates are not required
for the first release.

## 8. Open criteria

The following need product decisions before their detailed acceptance behavior
can be finalized:

- Detailed report and goal/projection comparisons beyond the specified MVP.
  All exports are deferred.
- First-slice handling of manually entered foreign-currency expenses before a
  confirmed CLP value is available. Preserve the original amount and do not
  invent a transaction-date estimate; decide how CLP category activity and
  availability are represented without silently counting zero spending.
- Later-release FX range calculation method/source, historical lookback
  settings and trend treatment.

### FX-R1: Later-release range display

Display semantics are accepted for the later FX scenario/range capability.
It is outside the MVP; the floating-cost funding-gap warning in AC-3.11 remains
an MVP requirement.

Given a CLP 1,000,000 target and USD 500 in selected expected backing resources,
when 900–1,000 CLP/USD scenarios are shown, then display CLP 450,000–500,000
coverage, or 45–50% scenario coverage of the target, alongside the goal's fixed
CLP funded progress. Keep these measures distinct. Identify which resources
back the comparison; showing them does not earmark money or independently
fund every goal sharing those resources.

Given foreign future costs and held foreign assets, then higher FX raises
CLP costs and asset values; use the same rate scenario when comparing both.
Count each wallet balance once. No range update changes actual balances,
recorded spending, assignments, goal funding/availability or the existing
floating-estimate funding-gap behavior.

A configurable historical lookback, such as six months or a year, and
trend-informed future ranges remain candidate calculations for the later
release. Source, method and settings remain undecided. Label historical observations
and future estimates distinctly, without guaranteed future bounds or a
promised provider integration.
