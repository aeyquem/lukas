# Conceptual domain model

Status: Working domain model derived from the current decisions in
`03-spec-v1.md`, `04-mvp-walkthrough.md`, `05-mvp-acceptance.md` and
`06-mvp-backlog.md`, with rewrite constraints from
`00-rewrite-context.md`. It connects product behavior to concepts,
relationships and rules before implementation. Acceptance references link to
[the MVP checklist](05-mvp-acceptance.md).

This is not a database, API or architecture design. The Rust/web and
self-hosting direction remains context. Earlier discovery descriptions of
pool reservations, separate pool funding and automatic funding from the
purchase account are superseded. Simple priority fallback and a current
net-worth snapshot are MVP behavior; resource restrictions and unlock events,
historical combined net-worth trends and all exports are later scope.

## 1. Four meanings that must stay separate

| View | Question it answers | Concepts | What changes it |
| --- | --- | --- | --- |
| Actual money and debt | What do I hold or owe, and where? | Accounts, dated opening balances, confirmed account activity, investment values | Confirmed financial activity and explicit corrections/adjustments |
| Budget intent | What job has available money received? | Ready-to-assign, category assignments, specifically funded goals, shared savings, card bill reservations | Budget assignment, spending, carryover and budget corrections |
| Reporting context | In which view or period should activity appear? | Actual date, assigned reporting period, statement close/due context, valuation observations | A report's chosen basis and relevant observations |
| Forecast | What may happen under these assumptions? | Recurring occurrences, historical estimates, scheduled purchases, projected pool balances | Projection assumptions, linked actuals and simulated activity |

These views describe related money; they are not four sources of money. An
assignment does not change an account balance. A pool is a view of accounts,
not another asset. Assigning a reporting period does not change a transaction
date. A forecast cannot change actual balances or monthly assignments.

The personal financial context keeps its accounts, sources, decisions and
plans together. The rewrite requires a workspace boundary even for one person;
this model does not decide membership or collaboration behavior.

## 2. Actual financial activity

### Accounts, balances and reconciliation

An **account** represents a tracked place holding money or debt: bank, cash,
savings, credit card or investment. It has a name, type, base currency and
budget eligibility. A physical bank/card account may contain multiple
**currency balances/wallets**, each retaining its own denomination. The base
currency is distinct from reporting currency; a displayed base-currency total
does not establish an extra balance. Budget eligibility and pool grouping
remain choices for the account, not wallet-level settings. An institution,
account reference, notes and pool are optional. Budget eligibility affects budgeting, not whether its activity can
be categorized or reported. Bank, cash, savings and cards default to
budgetable; investments default to tracking only. A credit limit is never
money. (AC-1.5, AC-5.2)

A **dated opening balance** is explicit opening-adjustment activity with
provenance. Its amount is confirmed deliberately, including zero. It is not
ordinary income or an unexplained mutable starting value. A calculated
account balance follows the opening balance and confirmed activity after
its date; the opening date identifies the boundary of tracked history.
Earlier activity must not be silently added on top of a balance that already
includes it. For a manual transaction dated before the opening date, require
the accepted per-wallet apply-effect or preserve-current-balance choice. When
it has the same date and inclusion is unknown, require the same initially
unset choice; do not infer chronology. Same-day and pre-opening imports remain
later scope. (AC-1.1–1.2)

An **account-side effect** describes whether activity increases or decreases
an account's money or liability, separately from its business meaning. For
example, an expense decreases bank money but a card purchase increases debt.
An amount retains its currency and direction; a sign alone does not explain
whether it is income, expense, transfer, refund or adjustment.

A **reconciliation** compares a calculated balance with an observed real
balance at a known date. The difference remains visible while the user reviews
and matches activity, makes an explicit adjustment, or leaves it unresolved.
A reconciliation adjustment is labeled account activity with source/history
and an explained ready-to-assign effect; it is not ordinary category spending.
Reconciliation does not silently overwrite prior activity. It applies to every
account type. (AC-1.3)

A basic **investment value** is a dated account value, with variation from
the prior period. Value changes must not be mistaken for spendable income or
new budget funding. Investment holdings and transaction-level activity remain
later scope; how values are updated and which comparison period applies are
still open. (AC-1.4)

### Transactions, portions and account effects

A **transaction** is one logical financial activity used by balances and
reports. Confirmed activity has an account, actual business date, amount and
meaning. Payee, category and note can be absent. Transactions retain manual or
imported provenance and correction history. Confirmation is a user trust
decision; it is independent of whether a bank has posted the activity.
(AC-2.3–2.4, AC-2.8)

| Meaning | Account relationship | Spending/budget meaning |
| --- | --- | --- |
| Income | A receipt into an account | Eligible budget funding is counted once, subject to account eligibility |
| Expense, including a card purchase | Decreases held money or increases card liability | Category spending once; may also be goal usage |
| Refund | Return of a prior expense amount | Reduces spending in its category; an original purchase link is optional |
| Exposed fee | Separate account activity when the source exposes it | Defaults to the related purchase's category and remains reviewable |
| Transfer | Connects both sides of movement between owned accounts | Neither income nor spending; changing location is not goal funding |
| Opening/reconciliation adjustment | Explicit dated account correction | Distinct from ordinary income/category spending; budget effect is explained |

A **split portion** assigns part of one expense to a category. Portions
exhaust the expense amount without overlap in its original currency; they do
not add account outflows. A category-filtered total includes only matching
portions once, retaining the parent transaction for inspection. Tags are
multiple cross-cutting labels, separate from category and goal relationships.
An uncategorized portion stays visible. (AC-3.2, AC-3.8)

A **transfer** has a source and destination owned account and an amount on
each side. A cash withdrawal is a transfer to a cash account; later cash
spending is a separate expense. Cross-currency transfer amounts remain in
their actual account currencies. Their numeric values need not be equal;
conversion observations explain their relationship. Importing evidence from
both accounts must not create two logical transfers. Separately exposed fees
remain separate expenses; detail cannot be invented from a final charged
amount. (AC-3.6; rewrite provenance constraints)

### Cards, statements and bill reservations

A **card purchase** is spending when made by default, even though held bank
money has not yet left. A **card payment** is a transfer from an owned
money-holding account that reduces the card liability, never another purchase.
(AC-3.3)

A **statement** supplies a close date, due date and amount due, with source
context. The card's configured closing/due days support these periods and
reminders. Prefer the statement amount due, allowing user correction; otherwise
calculate it from recorded activity through closing. A reminder's read state
is separate from payment coverage. Recorded payments covering the amount due
clear the payment reminder; marking it read does not mark it paid. Lukas
records these facts and does not initiate payments. (AC-3.7)

A **bill reservation** is genuinely cash-covered budgeted money already
consumed from a purchase's category but retained for the later card payment.
It is neither extra money
nor another goal/category assignment. Existing card debt reduces initial
ready-to-assign; a later budgeted purchase uses category funding and creates
its reservation once, rather than deducting debt again from ready-to-assign.
Payment uses available reservation without another expense or assignment
for the covered amount. The category impact and reservation follow the same
resolved budget month. (AC-5.6, AC-5.8)

**Uncovered card debt** is the purchase portion without a genuinely cash-covered
bill reservation. A negative category signals its unfunded use; an unsupported
positive assignment does not prove coverage.

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

## 3. Money, dates and valuation observations

Every monetary amount has a currency; amounts and exchange rates require
decimal-safe arithmetic. Distinguish the **original purchase amount** in the
merchant's currency, the **account-side amount** affecting money/debt in the
affected balance's currency, and the **report valuation** used to present
activity in CLP. These amounts can differ even within one bank/card account.
Both sides of cross-currency transfers are likewise retained.
**Amount precision and residual allocation:** ordinary CLP amounts use whole
pesos and USD amounts use cents for entry/display. Retain higher supplied
precision and source rates for inspection. Calculate derived portions first,
then round them; authoritative bank purchase amounts are not replaced by
internally rounded allocations. Rounded portions exhaust the authoritative
total exactly. Allocate leftover smallest units to the largest fractional
remainders, breaking ties by displayed portion order during review. Show the
remainder distribution and permit valid manual changes that preserve the
total. Later display sorting does not change confirmed allocations. Refunds
follow the same treatment with their signs retained; rounding does not create
another expense/account effect. For example, CLP 1,000 equal thirds become
334 + 333 + 333, and USD 1 equal thirds become 0.34 + 0.33 + 0.33. (AC-3.12)

A **CLP valuation observation** explains a
reported conversion rather than replacing the original amount. Its context
includes the amount/rate, source, relevant date, and whether it is an estimate,
final statement amount, correction or reconciled value.

One financial activity may have several observations: a pending estimate, a
final billed amount and a later correction. For spending, prefer the final
CLP statement amount; otherwise use a transaction-date estimate and show its
source/rate. A new observation does not create another expense. Historical
estimates remain inspectable so a changed rate does not silently rewrite the
explanation of an earlier report. (AC-2.4, AC-3.5)

An **actual bank conversion** relates input/output amounts in two currencies,
its source, conversion date and rate when available or derivable. A chain can
connect an original EUR purchase to USD debt, then to CLP debt at billing-cycle
end. Direct JPY/EUR-to-CLP conversion or retaining the spent currency are also
valid bank behaviors. The purchase date remains separate from conversion
dates. Each actual conversion replaces the converted portion's denomination;
it does not leave both old/new debt outstanding, create another expense or
imply an unobserved fee. (AC-3.9)

A report-only revaluation is not an actual bank conversion. Without evidence
of conversion, retain the balance in its actual currency. Pending/provisional
account amounts stay labeled. Final confirmed evidence replaces the prior
estimate or applies its difference once, retaining history and the existing
closed-budget/current-period correction process. Reconciliation compares
separately denominated balances. Current rates may change a foreign balance's
CLP display valuation, but cannot overwrite a fixed actual CLP debt or prior
bank conversion. (AC-2.4, AC-3.5, AC-3.9)

A **derived conversion allocation** attributes an authoritative aggregate
bank conversion to known components of the converted balance. It is distinct
from bank-supplied purchase-level conversion amounts, which take precedence.
Its reviewable input is the eligible outstanding composition actually included
in the conversion; the aggregate's effective ratio does not revalue every
historical purchase. Suggest proportional allocation for known portions,
require confirmation, allow manual changes, and reconcile totals with any
rounding discrepancy visible. Partial conversions apply only to converted
portions; already-settled portions remain unchanged. (AC-3.10)

Each component keeps its meaning: purchase/category/goal usage valuation can
change once; exposed fees and refunds retain their links/classification;
opening debt is not fresh spending; payment coverage allocations do not create
another transfer or expense and require review when uncertain. Goal funding
does not increase automatically. Retain original dates and allocation history,
with the existing closed-budget/current-period correction rules. Incomplete
composition can leave purchase valuations provisional while the actual
aggregate debt conversion is recorded; never invent missing components or
bank-supplied purchase amounts. (AC-3.10)

A **foreign cost estimate** can be locked to a selected dated rate or float
with a selected current rate and optional user spread. An available selected
card-network rate is another possible source, without an integration
commitment. Foreign future cost components affect forecasts and estimated
funding gaps, not actual spending or the CLP denomination of goal targets.
After purchase, the real foreign debt and recorded initial provisional CLP
expense remain distinct from this cost estimate. A floating estimate changes
the estimated funding gap only, not recorded spending, goal usage, remaining
amounts, funding or reservations. A purchase-day lock remains provisional;
actual bank conversion/reviewed allocation updates the original expense once
under the existing hierarchy and correction rules without new funding.
Significant-change alerts do not create financial updates; their threshold
policy remains open. (AC-3.11, AC-5.9)

Current account, pool and net-worth views answer a different question from
historical spending. They show original-currency balances with balance dates
and current CLP estimates with conversion source, rate and date. Missing
conversion data is unavailable, never zero. Current net worth includes tracked
assets less liabilities, including accounts excluded from budgeting. Pool
holdings, goal balances and budget assignments are not added to net worth
again. Each separately denominated balance contributes once; a displayed
base-currency summary is not an extra balance. Converted portions are no
longer outstanding in the replaced denomination. Historical combined net-worth
trends await a valuation rule.
(AC-4.2, AC-3.8)

A **business date** is the date the financial activity occurred. An **assigned
reporting period** is an interpretation for a view, with card close/due context
or a manual assignment. Each report chooses its basis; each monthly budget
chooses actual-date or assigned-period basis independently. Moving a purchase
to another reporting period never moves its actual account effect or changes
its real date. Audit timestamps describe when an entry, review or correction
was recorded, separately from business dates. (AC-3.4, AC-5.6)

A **resolved budget month** identifies the one monthly budget affected by a
purchase. Each budget's chosen basis supplies defaults; when mixed settings
would include a purchase twice or omit it, the user resolves attribution to
one month. Category impact and a card purchase's bill reservation use that
same month. A report's independent date basis does not change this attribution.
Later basis or attribution changes must not silently duplicate or drop impacts;
closed snapshots and the agreed correction process remain in force. (AC-5.8)

Actual-date cashflow shows a card payment in its actual payment month as a
payment/transfer, with the bank outflow and card-debt reduction. The payment
creates no additional expense and does not move the purchase's budget month.
(AC-3.3, AC-5.8)

## 4. Source evidence, review and corrections

A **source record** preserves original evidence from manual entry or a file,
with its raw and normalized interpretation. An **import batch** identifies
the import attempt, selected account, worksheet where applicable, and file-wide
interpretation choices. A **source link** connects evidence to the logical
transaction it supports. Several source records can support one transaction;
new evidence does not by itself imply new activity.

Source facts remain intact. Linked, discarded, superseded and needs-review
meanings belong to links, decisions and review history rather than rewriting
the original source fact. Corrections to interpreted values preserve what the
file said. External identifiers are matching evidence rather than permanent
financial identity. Repeated/overlapping imports must respect prior link and
discard decisions instead of resurrecting activity. (AC-2.2, AC-2.7–2.10;
rewrite source constraints)

A **review item** describes what requires a user decision: malformed or
ambiguous values, an unresolved transfer counterpart, likely duplicate
activity, or changed source evidence. A **review decision** records what the
user accepted, changed, linked/merged, kept separately or discarded, with the
supporting source and previous interpretation.

The import lifecycle is:

1. Select one account and, for Excel, a worksheet; preview raw columns and
   interpreted values using explicit mapping and file-wide date/number/sign
   choices. A file-level failure confirms no rows.
2. Retain reviewable source rows. Required interpreted date, amount and
   meaning must be resolved; a transfer also needs the other owned account.
   Missing category, payee or note does not block confirmation.
3. Confirm selected valid rows individually or in bulk. Unresolved rows remain
   in review. Likely duplicates require an individual keep, link/merge or
   discard decision outside bulk confirmation.
4. Confirmation creates or links financial activity once. Only confirmed
   activity affects calculated balances, spending and actual-based totals.

A source row's file-selected account association remains inspectable when
any interpreted transaction field is corrected. Manual entry also has
lightweight provenance; it need not masquerade as an imported file.
(AC-2.1–2.2, AC-2.8–2.10)

A **confirmed pending** card transaction is valid actual activity with an
estimated amount, not an unreviewed import. Posted evidence updates that same
activity's effective spending amount while preserving pending and posted
context. Confirmation and posting are separate lifecycles. (AC-2.4)

A **correction** edits the transaction directly and retains prior details and
the user's change. Actual dates remain intact rather than being shifted to
the correction date. Account balances and transaction reports reflect the
correction. When a closed budget is affected, its close snapshot remains
intact and a linked current-period budget adjustment applies the impact where
possible. That budget adjustment is not a second account transaction or
expense. (AC-2.7, AC-5.7)

## 5. Budget assignments, goals and pools

### Monthly budget intent

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

Eligible foreign money held (assets) automatically receives daily mid-market
CLP budget valuation adjustments in the MVP. Explain the FX difference
separately from ordinary income/spending with rate/source/effective date;
positive changes add
ready-to-assign once, while decreases may expose over-assignment without
changing fixed assignments, funded goals or targets. No lock or forced
reassignment follows. Current account/report valuations also update on
tracking-only assets, without budget funding effects. Preserve actual wallet
currencies and known bank conversion amounts; a CLP estimate is not a bank
conversion. Do not count existing receipts/transfers/transaction effects or
prior valuation adjustments again. Preserve closed snapshots and apply current
effects under the existing correction rules. On weekends and holidays, use the
latest published daily rate and show its effective date. Frankfurter v2 is the
selected provider. If it is unreachable, use the latest stored observation and
mark it stale; if none is stored, mark the foreign valuation unavailable
without treating it as zero or returning a partial CLP budget-funding total.
(AC-5.10)

This asset-funding policy is separate from warning-only floating expense
estimates and from current foreign-debt valuation for net worth. Foreign-debt
coverage follows the warning-only rule in AC-5.17. User choice between automatic
and reviewed asset revaluation is later
scope; automatic is the MVP behavior. (AC-3.11, AC-5.10)

MVP budget assignments, category/goal funding, goal targets and monthly
contribution plans use CLP. Foreign-currency budget and goal-target choices
are later scope. A planned cost component may nevertheless retain an amount
such as USD 2,000 and show its estimated CLP cost; that component is not a
USD-denominated goal target.
(AC-5.9)

A **monthly budget period** is a calendar-month plan with a chosen period
basis, assignments, spending impacts, card reservations and carryover decisions.
**Ready-to-assign** (also called available-to-assign) is eligible money without
a job. It starts with eligible existing money once, after the agreed existing
card-debt reduction; later eligible income enters once. Re-reading the updated
account balance must not create another receipt. Income already received
can fund a later monthly budget once; money assigned for that later month
cannot remain simultaneously available for another assignment. (AC-5.1–5.2,
AC-5.8)

**Signed opening funding** counts actual budgetable balances according to
their economic sign. Negative bank/cash/savings balances reduce funding;
actual positive card credits add funding. Unused credit and overdraft limits
add nothing. For example, CLP 100,000 bank money, CLP -20,000 in another bank
and an actual CLP 5,000 card credit provide CLP 85,000 before assignments.
Preserve the opening card-debt deduction and count later transactions,
purchases/category impacts and bill reservations once, without deducting
debt twice or automatically treating card credit as new income. Eligibility
still controls budget participation. Credit consumption follows AC-5.18;
debt-bearing eligibility changes follow AC-5.19. Purchase coverage follows
AC-5.14. (AC-5.13)

A **budget assignment** gives money a job in a spending category or a
specifically funded goal category. A shared savings category is also a budget
destination. Pools are not destinations. Assignment changes ready-to-assign
and category/goal funding, without changing physical money. A reassignment
releases a job and gives that money another job; it is not new income.
Unassigned money can remain, and over-assignment can be saved with its explicit
shortfall warning. A recorded assignment therefore does not guarantee physical
coverage. (AC-4.6, AC-5.3–5.4)

A **budget-boundary funding effect** explains an owned money transfer that
crosses between budgetable and tracking-only accounts. Tracking-only to
budgetable adds funding once; budgetable to tracking-only removes it once.
Between two accounts in either group, there is no budget-boundary funding
effect. Independent FX differences and exposed fees retain their existing
treatment, counted once. The financial movement is still a transfer, not
income or spending. Keep assignments and
funded goals fixed; a funding reduction can expose over-assignment without
new category spending, a transfer category or automatic goal usage. Retain
actual dates and the established closed-period correction rules. (AC-5.11)

Payments of already-recognized debt follow AC-5.19: covered portions consume
protection, while uncovered portions retain the existing paid-uncovered gap
treatment. Neither portion receives another ordinary boundary-outflow
deduction. Unrelated asset transfers still follow AC-5.11.

For example, CLP 100,000 funding and CLP 80,000 assigned leave CLP 20,000
ready to assign. An owned CLP 30,000 transfer into a tracking-only account
leaves CLP 70,000 funding against the same CLP 80,000 assignments, exposing
CLP 10,000 over-assigned. Total owned money is unchanged; it has crossed the
budget boundary. (AC-5.11)

An **eligibility change** is a previewed and confirmed prospective change to
which account resources fund the budget. From confirmation onward, enabling
budgetability adds the current eligible positive asset balance once; disabling removes it
once. It changes no actual balance, bank source data, real transaction date
or independent categorization/report history. It creates no transfer,
income/spending, category use or goal usage. Keep assignments and funded
goals fixed; a funding decrease can expose over-assignment. Preserve closed
budget snapshots and do not replay original receipts or prior FX adjustments.
Existing foreign-asset valuation rules continue to apply; recognized debt and
coverage survive eligibility changes under AC-5.19. (AC-5.12)

Example: CLP 100,000 funding less CLP 80,000 assigned leaves CLP 20,000
ready to assign. Enabling a tracking-only account with eligible CLP 50,000
adds that amount once, giving CLP 150,000 funding and CLP 70,000 ready to
assign. No actual money moved or new income occurred. (AC-5.12)

**Retained debt responsibility** keeps recognized liabilities and their
budget effects separate from positive-money eligibility. Making a debt-bearing
card or negative-bank account tracking-only stops positive assets from funding
prospectively but preserves recognized debt protection, purchase reservations
and uncovered amounts until settled under existing rules. It releases no
ready-to-assign, reverses no spending and deducts no liability twice. Enabling
participation recognizes only previously unrepresented debt once under
opening-debt rules, without inventing cash-backed coverage or repeating a
prior category impact. Preserve genuinely funded protection where present;
otherwise expose the shortfall. (AC-5.19)

Payments of already-recognized debt keep their once-only treatment even
after the receiving account becomes tracking-only. Covered portions consume
protection; uncovered portions retain AC-5.15's paid-uncovered gap treatment.
Neither portion also incurs the ordinary budget-boundary outflow deduction
in AC-5.11. Unrelated asset transfers and positive-credit portions keep their
existing meaning. Preview/confirmation,
fixed assignments, no hard limits, source/actual-date/closed-snapshot rules
and existing FX/payment/refund/positive-credit policies remain in force.
For example, bank CLP 100,000 less already-recognized CLP 95,000 protection
for USD 100 debt leaves CLP 5,000 ready to assign. Making that card tracking-only
leaves both the protection and CLP 5,000 unchanged; the covered payment consumes
protection once. Likewise, turning a recognized CLP -20,000 bank balance into
tracking-only cannot free CLP 20,000. In a separate case, debt CLP 1,000
with CLP 600 reserved and CLP 400 uncovered becomes tracking-only. Paying
CLP 800 leaves no reservation, CLP 200 paid-uncovered and CLP 200 unpaid
uncovered debt. Those portions explain the original CLP 400 gap; no additional
CLP 200 boundary deduction is created. (AC-5.19)

The budget lifecycle is **open → explicit close → closed snapshot plus
subsequent correction history**. Calendar month-end does not close a budget.
Open periods accept dated additions and edits after the month ends. On explicit
close, each computable signed category remainder contributes once to the next
month's calculation. Until the source period is closed, provisionally carry
each signed remainder in the same category into the next period. At explicit
close, the user chooses per category whether to keep carrying it or release it
to ready-to-assign (AC-5.5); preserve negative signs and do not reseed the same
funds from account balances. Block close while required values are incomplete;
goal/pool consequences and other transition details remain open. The basic
close transition is included in the first slice.
Closed does not mean transaction lock. (AC-5.7)

### Specifically funded and shared-fund goals

A **goal** describes a target/cost, optional target date and expected payment
pool. There are two funding relationships:

- A **specifically funded goal** behaves as its own budget category with a
  target. Assignments fund it directly. Purchases use that category regardless
  of payment account. Its net **saved/funded** amount includes funding already
  used, after explicit reassignments and other approved reductions; **used**
  records ordinary goal-related spending; **available**
  is saved minus used. First-release progress is saved divided by target.
- A **goal-to-buy** points to a shared budget category. Its cost is compared
  with that category's available balance without an individual reservation.
  Multiple candidates can be individually affordable while their combined
  cost exceeds the shared balance. It shows cost, affordability and usage,
  not an individually saved amount or funded-progress bar.

Recording goal usage and categorizing the purchase describe the same expense.
The MVP includes a **manual purchase-portion-to-goal link**, a dedicated
relationship separate from tags. Each portion may link to at most one goal;
an unsplit purchase has one portion. Ordinary/shared-category links are
optional, but a specifically funded goal category requires the matching goal
association/effect. For a shared-category portion, the link
identifies which goal-to-buy receives usage; the category alone cannot
distinguish candidates. A split purchase may therefore link different portions
to different goals without multiple parent purchases. (AC-5.22)
For a specifically funded purchase, the goal link and its goal category must
describe the same expense consistently. Adding the link records no second
expense, category deduction or funding contribution. Automatic transaction-to-
goal suggestions remain later scope.

For a shared goal, that expense consumes the shared category once. For a
specifically funded purchase, the user chooses existing funding or additional
budget assignment spent immediately. Both steps may appear in one flow but
remain separately inspectable. Additional funding requires ready-to-assign,
explicit reassignment or an acknowledged over-assignment; paying from another
account/pool does not supply it automatically. (AC-4.5–4.9)

An ordinary purchase increases used without reducing cumulative saved. An
explicit **exceptional saved reduction** reduces saved instead of increasing
used; existing used remains and availability is recalculated. The exceptional
choice is auditable. Ordinary linked usage refunds follow AC-5.20;
exceptional/excess refunds still need the refinement in section 9. Explicit
funding releases follow AC-5.21. (AC-4.11)

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
This is the warning policy for explicit funding reassignments only; exceptional
saved-reduction transactions/refunds remain separate questions. Correcting a
purchase-portion goal link follows AC-5.22. (AC-5.21)

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

**Greedy allocation** assigns budget money in goal priority order, passing
remaining allocatable money onward. **Paced allocation** assigns a monthly
amount per goal. A dated goal shows required saving per calendar month; an
undated paced goal can have a user-set monthly amount. Each goal chooses
automatic allocation or suggestions requiring confirmation. An unconfirmed
suggestion is not funding. Neither strategy reserves pool holdings.
(AC-4.10, AC-4.12)

### Pools as expected payment sources

A **pool** groups whole money-holding accounts by purpose in the MVP. Account
pool membership is optional; an account belongs to at most one pool. Several
goals can identify the same expected pool, and actual purchases identify the
accounts actually used. Cards can supply transaction context for a pool but
their debt and available credit never increase pool money. (AC-4.1, AC-4.3)

Pool holdings are the grouped accounts' money, with original currencies and a
current CLP estimate. The pool also exposes calculated resource availability,
a separate user-set spendable amount, and a choice whether that manual amount
is display-only or used for planning. Budget-designated funds remain distinct.
The informational sum of linked goal targets reserves nothing. Pool payment
coverage and shared-category affordability answer separate questions; neither
is extra funding to add to a goal. The detailed resource-availability/manual
amount interaction remains open rather than inferred from old reservations.
In projections, a known dated obligation changes simulated pool resources on
its occurrence date, not upfront; count the projected outflow once. Linked
goal targets are informational and do not count as obligations.
(AC-4.2, AC-4.4–4.5, AC-4.9)

### Later-release FX coverage and cost ranges

A **scenario coverage range** compares selected expected backing resources
with a CLP goal target alongside its separate fixed budget-funded progress.
It does not give those resources another job, earmark them or create individual
funding for all goals that share them. For a CLP 1,000,000 target and USD 500
of selected backing, 900–1,000 CLP/USD scenarios yield CLP 450,000–500,000,
or 45–50% scenario coverage of the target. Actual funded progress retains
its separate CLP budget meaning. (FX-R1)

A foreign future cost may also have a **scenario cost range**. Higher FX
increases both CLP foreign-cost estimates and held foreign-asset values; compare
both with the same rate scenario and count each wallet balance once. Ranges
are informational and do not update actuals, recorded spending, assignments,
goal funding/availability or the existing floating-estimate gap policy.
The FX range view is later scope; the floating-cost funding-gap warning
remains MVP scope. Historical lookback
settings such as six months/a year, observed min/max versus trend-informed
future ranges, method and sources remain open for the later release; no guaranteed future bounds
or provider integration follows. (FX-R1)

## 6. Recurring occurrences and projections

A **recurring expense rule** supplies a schedule and estimate. Supported
schedules are every N days, a selected day of month, month-end and an ordinal
weekday. It can end on a chosen date. A **forecast occurrence** is a dated
estimate generated by the rule; its amount can be changed independently, it
can be skipped or rescheduled, and it is never actual activity merely because
its date arrives. Multi-transaction event templates remain later scope.
(AC-6.3)

An occurrence can be linked to confirmed actual activity through a
user-confirmed match. The actual replaces that occurrence's estimate in the
projection; the relationship retains the estimate and actual for inspection.
A suggested match alone does not replace the estimate. Choosing recurring
rules versus history-based estimates for overlapping expenses is a separate
per-projection decision, so the same expected expense is not added twice.
(AC-6.4)

A scheduled goal purchase may overlap a recurring or history-based estimate.
The user decides whether the purchase and estimate represent the same expected
cost or separate costs. A same-cost pair contributes one projected cost; a
separate-cost pair contributes both. Until the user marks them as the same
cost, keep both separate and count both. Preserve the two sources and the
user's decision in the projection assumptions, and do not infer overlap
silently.
(AC-6.6)

A **projection** has a historical lookback, future horizon and visible
assumptions. It combines selected history estimates, recurring occurrences,
linked actuals, planned goal savings and scheduled goal purchases. It shows
estimated income, spending, net cashflow and dated-goal progress. Planned
saving reduces money left after expenses separately from spending; it is not
itself an account outflow or purchase. (AC-6.1–6.2)

A **scheduled goal purchase** draws down projected resources in its period.
Use its preferred pool first, then subsequent pools in spending-priority order;
show each contribution. Known dated obligations reduce the affected projected
pool balance when they occur, so only later purchases draw from the reduced
balance; do not subtract an outflow twice when it is already represented in the
projection. Reduced balances carry into later periods. A
**reserve target** is a desired retained amount and warning threshold, not an
extra purchase, budget assignment or hard floor. Fallback may cross it with a
warning. Drawdowns stop at zero and any remainder is unfunded. These operations
cannot change actual accounts or monthly assignments. User-defined restrictions
and unlock events belong to later planning behavior. (AC-6.5)

## 7. Rules that must always hold

1. **One fact, no duplicate money:** accounts hold money/debt; pools summarize
   it, assignments give it jobs and goals describe its purpose. These values
   are not added together as assets or funding. A source link, split, goal
   link or valuation observation does not create another movement.
2. **Confirmation gates imported actuals:** unresolved or unconfirmed rows
   affect no actual balance or report. Confirmation does not imply final
   posting, reconciliation or certainty of a conversion estimate.
3. **Purpose does not move money:** assignment, carryover and reassignment do
   not transfer accounts. Transfers, including card payments, are neither
   income nor spending and do not create a goal contribution.
4. **Count receipts and jobs once:** opening eligible money and each eligible
   income receipt enter ready-to-assign once; carried assignment is already
   assigned. A card purchase's category use and bill reservation are one use
   of budgeted money, not two assignments or two deductions for debt.
5. **Classification preserves the whole:** expense splits sum to the original
   expense; filtered reports count matching portions only. Goal usage and
   category spending refer to the same purchase amount, not additive expenses.
6. **Evidence survives interpretation:** source values, review decisions,
   pending/final valuation context and corrections remain inspectable. An
   unexplained difference requires an explicit adjustment or stays unresolved.
7. **Dates and views have distinct meanings:** reporting/budget period choices
   never rewrite real transaction dates. A closed budget snapshot survives
   later transaction corrections even when corrected transaction reports differ.
8. **Estimates remain estimates:** recurring rules, projected assignments,
   pool drawdowns and scenario assumptions cannot mutate actual balances,
   actual transactions or monthly budget assignments.

## 8. Worked checks against accepted behavior

All amounts below are CLP unless an original currency is named. These examples
illustrate the agreed rules, not a universal budget calculation formula.

### A. Opening money, income and carryover

Start with eligible bank money of 10,000, existing card debt of 2,000 and no
prior jobs. Ready-to-assign is 8,000; the card's credit limit contributes zero.
Assign 5,000 to Vacations and 3,000 to groceries: ready-to-assign becomes zero,
and the bank still holds 10,000. Receive eligible income of 4,000: the bank
becomes 14,000 and ready-to-assign becomes 4,000, not 8,000 by adding that
income again from the updated balance. (AC-1.5, AC-4.6, AC-5.1–5.3)

For a separate close example, 1,000 held in an eligible account is already
assigned and unspent in General Savings, with nothing else unassigned. Before
the source month closes, the next month provisionally carries that 1,000 in
General Savings. At explicit close, the user can keep it there or release it
once to the next month's ready-to-assign; it is not new income. The closed
month's snapshot remains intact.

### B. Funded progress and payment location

Vacations has a 10,000 target, saved 5,000 and used zero. A 1,000 ticket
categorized to Vacations using existing funding gives saved 5,000, used 1,000,
available 4,000 and progress 50%. Assigning another 1,000 before that same
purchase instead gives saved 6,000, used 1,000, available 5,000 and progress
60%. Either result can occur when paying from the expected savings pool or
from an account outside it; the funding choice determines the result.
(AC-4.7–4.9)

Starting again at saved 5,000, assign another 500 and transfer that 500 from
bank to savings. Then assign 1,000 and spend 1,000 on tickets. Saved is 6,500,
used is 1,000 and available is 5,500. The 500 transfer changes location; only
the 500 assignment counts as funding. A later card bill payment adds neither
usage nor funding. (AC-4.8)

For the exceptional choice, saved 5,000 and used 1,000 leave available 4,000.
An explicitly marked 500 saved reduction gives saved 4,500, used 1,000 and
available 3,500. It does not also increase used by 500. (AC-4.11)

For a specifically funded goal with saved CLP 5,000, used CLP 1,000 and
available CLP 4,000, an ordinary linked CLP 300 refund leaves saved CLP 5,000,
used CLP 700 and available CLP 4,300. Funded/target progress is unchanged.
For a goal-to-buy with CLP 1,000 used and CLP 4,000 available in its shared
category, the same linked CLP 300 refund leaves usage CLP 700 and shared
availability CLP 4,300, without an individual saved amount or duplicate
category restoration.
(AC-5.20)

For a goal with saved/funded CLP 5,000, used CLP 1,000 and available
CLP 4,000, explicitly moving CLP 500 to another budget category/goal leaves
funded CLP 4,500, used CLP 1,000 and available CLP 3,500. Moving CLP 4,500
instead from the same initial state leaves funded CLP 500, used CLP 1,000 and
available CLP -500. Warn about that single CLP 500 coverage shortfall; the
receiving CLP 4,500 assignment does not prove that all of it has backing.
(AC-5.21)

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
(AC-5.22)

### C. Shared savings and split reporting

General Savings has 4,000 available. Goal A costs 3,000 and goal B costs 2,000.
Each is individually affordable; together they need 5,000, a 1,000 conflict.
Buying A records one 3,000 expense against General Savings and 3,000 usage
against A. The shared category has 1,000 left; B is no longer affordable.
No 4,000 individual saved balance was created for either goal. (AC-4.5)

A separate 1,000 expense split into groceries 600 and transport 400 changes
its account by 1,000 once. Filtering groceries shows 600 and the inspectable
parent; showing the parent does not add another 1,000. A categorized 100
grocery refund then makes grocery spending 500. (AC-2.5, AC-3.2, AC-3.8)

### D. Card spending, reservation and payment

With no existing debt, a bank holds 10,000 and groceries has a 2,000 budget
assignment. A confirmed 1,000 card purchase leaves bank money at 10,000,
increases card debt by 1,000, uses 1,000 of groceries and reserves that 1,000
for the bill. It creates no additional ready-to-assign reduction. The later
1,000 bank-to-card payment leaves bank money at 9,000 and clears that debt and
reservation. Total spending is still 1,000 and no second assignment is needed.
(AC-3.3, AC-5.6)

For a separate purchase with CLP 600 genuinely available in a category,
a CLP 1,000 card purchase records CLP 1,000 spending and debt, leaves category availability
at CLP -400, reserves CLP 600 for the bill and shows CLP 400 uncovered debt.
Before any payment, a later explicit CLP 400 assignment/reassignment backed
by eligible money brings category availability to zero and the reservation to CLP 1,000;
spending and debt remain CLP 1,000. Merely entering an unsupported CLP 400
assignment does not cover that debt or increase the cash-covered reservation.
(AC-5.14)


For another separate case, start with CLP 1,000 debt, CLP 600 reserved,
CLP 400 uncovered and category availability CLP -400. A CLP 500 payment leaves
CLP 500 debt, CLP 100 reserved and CLP 400 uncovered. Starting again from
the original state, a CLP 800 payment leaves CLP 200 debt, zero reservation,
CLP 200 paid without prior coverage and CLP 200 still-unpaid uncovered debt.
Those last two portions are the original CLP 400 gap. Funding it later with
CLP 400 covers CLP 200 already paid and reserves only CLP 200 still owed;
category availability becomes zero without changing spending or debt.
Payments remain transfers in their actual month, independent of the original
purchase's resolved budget month. (AC-5.15)

Starting with a CLP 1,000 purchase funded with CLP 600 in its original
category, CLP -400 original-category availability, CLP 600 reserved and
CLP 400 uncovered debt, a linked
CLP 300 refund leaves debt CLP 700, category availability CLP -100, reservation
CLP 600 and uncovered debt CLP 100. No reservation is released yet.
Starting instead from that same original state, a CLP 500 refund leaves debt
CLP 500, category availability CLP 100, reservation CLP 500 and no uncovered
debt. Both cases reduce spending once without income or new ready-to-assign.
(AC-5.16)

For a same-currency example, bank CLP 700 plus actual card credit CLP 300
provide CLP 1,000 funding. Assign CLP 500 to Groceries, leaving CLP 500 ready
to assign. A CLP 500 card purchase uses the credit CLP 300 and creates only
CLP 200 debt: bank stays CLP 700, Groceries availability becomes zero,
reservation is CLP 200 and ready-to-assign stays CLP 500. Spending is CLP 500
once; no extra funding or assignment is created from the consumed credit.
(AC-5.18)

If that purchase's actual date is 28 October and its assigned period is
November, the actual-date spending report shows October. A purchase resolved
to November applies category use and reservation there once, even if October's
actual-date basis and November's assigned-period basis both nominated it.
The inverse basis combination also requires resolution rather than omitting
it. The account history still retains 28 October. Eligible income received in
October can fund November once, without making the same money available for
another October assignment. A 10 November card payment appears in November's
actual cashflow as a transfer with bank outflow and debt reduction; it creates
no second expense and does not determine the purchase's budget month.
(AC-3.3–3.4, AC-5.6, AC-5.8)

### E. Evidence, valuation and late correction

A USD 10 card purchase first has a confirmed pending CLP estimate of 9,000.
Final statement evidence says 9,200. Spending becomes 9,200 once, with USD 10
and both observations inspectable. A repeated imported row is held for review;
linking it as more evidence adds no second purchase. An unconfirmed unrelated
row remains outside balances/reports. (AC-2.1–2.4, AC-3.5)

A transaction in a closed October budget is corrected from 1,000 to 1,200.
Corrected account and actual-date spending history show 1,200 on the original
date. October's closed budget snapshot retains its prior 1,000 impact. Where
applicable, the current budget receives the linked 200 correction impact; no
second 200 account outflow is created. The fallback when current-period
application is not possible remains open. (AC-2.7, AC-5.7)

### F. Actual currency conversion chain

A card's base currency is CLP, but it can hold USD debt. A EUR 100 purchase on
5 October becomes USD 110 at the bank's conversion of 1.10 USD/EUR. A CLP
104,000 spending estimate does not create an additional CLP debt alongside
the actual USD 110. On 25 October, the bank converts that USD 110 at 940 CLP/USD
into CLP 103,400. USD debt for that portion becomes zero and CLP debt becomes
103,400. Spending uses the final attributable CLP 103,400 once, decreasing its
prior estimate by 600 with the agreed review/correction process. Original
purchase facts and both conversion steps remain inspectable. Current net
worth values whichever debt is actually outstanding; a later USD rate does
not change the now-fixed CLP 103,400. (AC-3.5, AC-3.9)

A JPY 10,000 bank purchase converted directly to CLP 60,000 records that one
actual conversion (6 CLP/JPY); it does not invent intermediate USD debt. If
no bank conversion occurs, retain the actual JPY balance effect and use CLP
only as a labeled report valuation. Eligibility/pool grouping stay with the
account in each case. (AC-3.9)

### G. Aggregate conversion with mixed debt

A bank converts USD 100 to CLP 90,000, with no purchase-level CLP amounts.
Known outstanding composition is USD 60 of tracked Vacations purchases and
USD 40 of opening debt. The default suggestion allocates CLP 54,000 and
CLP 36,000, labeled derived, requiring confirmation and allowing manual
changes. The bank conversion remains authoritative; the derived allocation
does not claim that the bank supplied those purchase amounts. If the purchase
estimate was CLP 53,000, its confirmed spending/goal usage valuation increases
by CLP 1,000 once; saved/funded does not automatically increase. The CLP 36,000
opening-debt portion creates no new expense. (AC-3.10)

For a separate USD 50 partial conversion from the same known mix to
CLP 45,000, suggest USD 30/CLP 27,000 for the purchases and USD 20/CLP 18,000
for opening debt, leaving USD 50 outstanding. Do not reallocate previously
settled portions. Incomplete composition leaves purchase valuations provisional
for review without blocking the actual account conversion. (AC-3.10)

### H. Forecast replacement and pool fallback

A recurring occurrence estimates internet at 20,000. After the user confirms
a match to an actual 22,000 payment, the projection uses 22,000 for that
occurrence, not 42,000. Creating, skipping or rescheduling another occurrence
does not change an actual account balance. (AC-6.3–6.4)

Before a scheduled 3,000 goal purchase, projected preferred-pool resources
are 1,000 and the next pool has 1,500 with a reserve target of 1,000. Fallback
uses 1,000 then 1,500, leaving both projected balances at zero, an unfunded
500 purchase cost and a 1,000 reserve-target shortfall. Later periods carry
those zero balances; the original actual pool/account holdings and monthly
budget remain untouched. (AC-6.5)

### I. Variable cost estimate and actual conversion

A USD 2,000 hotel component belongs to a CLP plan. At 900 CLP/USD its estimated
cost is CLP 1,800,000, or CLP 1,836,000 with an optional 2% user spread.
Before purchase, changing that estimate affects the forecast/funding gap only.
After purchase, assume USD 2,000 actual debt, CLP 1,800,000 recorded provisional
spending/ordinary goal usage and CLP 1,800,000 funding. A floating cost
estimate of CLP 1,900,000 shows a CLP 100,000 gap; recorded usage remains
CLP 1,800,000 and available remains zero. Net worth independently values the
real foreign debt currently. A locked purchase-day estimate cannot prevent an
actual conversion from becoming authoritative. (AC-3.11, AC-5.9)

A confirmed actual CLP 1,880,000 conversion updates the original expense by
CLP 80,000 once: goal usage is CLP 1,880,000, available is CLP -80,000 and
funding stays CLP 1,800,000. Reservation funding and any closed-period impact
follow the existing rules. Eligible held foreign assets separately use the
automatic funding-valuation policy, not this cost-estimate setting.
(AC-3.11, AC-5.10)

### J. Automatic valuation of eligible held assets

An eligible account holds JPY 5,000, initially valued at CLP 30,000 and fully
assigned to a goal. At a current CLP 32,000 valuation, explain an FX valuation
adjustment of CLP 2,000 and add it to ready-to-assign once. Goal funding stays
CLP 30,000 unless the user assigns the extra. If the starting CLP 30,000 value
instead falls to CLP 28,000, the unchanged assignment exposes a CLP 2,000
funding shortfall/over-assigned warning, without a lock. JPY 5,000 remains the
actual asset balance; no income or bank conversion was created. Tracking-only
JPY would update current valuations without adding budget funding. Closed
snapshots remain intact; current effects and previous adjustments are counted
once. This does not revalue a foreign expense's recorded amount or reservation.
(AC-5.10)

### K. Foreign-debt current valuation and fixed budget coverage

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
(AC-5.17)

## 9. Product semantics still to refine

These questions arise from the accepted concepts; they are not alternative
funding models or reasons to defer the agreed MVP. Their eventual answers must
preserve the rules above.

- **Unvalued foreign expense in the first slice:** Show the category's CLP
  activity and remaining as incomplete, alongside the known CLP subtotal and
  unresolved original amounts. Dependent aggregate availability is incomplete.
  Filling the final confirmed CLP value completes the existing expense once,
  without another expense or debit. Ready-to-assign is complete only if its own
  funding inputs are complete, with an adjacent pending-expense warning.
  Assignments remain savable intent. Block close while required values are
  incomplete; never silently show zero. This behavior is accepted.
- **Foreign opening liability:** Preserve the signed original-currency debt
  and require a separately supplied signed CLP opening funding basis. Apply
  that basis once to initial funding; do not use daily debt FX for further
  funding deductions. If the basis is missing, funding and ready-to-assign
  are unavailable. When the foreign opening-liability amount is directly
  corrected, the user must supply or explicitly confirm its signed CLP funding
  basis in the same update; never derive the revised basis from daily debt FX.
  This paired-update rule is accepted. Basis storage and provenance, and how
  apply/offset corrections affect it, remain to specify.
- **Tracking boundaries and investment values:** For a manual transaction on
  the opening date with unknown inclusion, require an initially unset
  per-wallet apply-effect or preserve-current-balance choice; do not infer
  chronology. This manual-entry rule is accepted; same-day and pre-opening
  imports are later scope. How are investment values updated and their
  comparison periods selected?
- **Bank attribution:** Purchase coverage, partial payments, linked refunds
  and positive-card-credit consumption are settled in AC-5.14–5.16 and
  AC-5.18, including existing opening-debt protection and uncovered-first
  refund ordering. Returns of owned money retain transfer treatment. Ordinary
  goal-linked refunds follow AC-5.20; remaining goal cases are listed below.
  Attribution of actual payments to
  specific bank purchases/statements still needs appropriate evidence; do not
  infer bank allocation from the budget's coverage-first bookkeeping.
- **Goal changes:** Ordinary linked purchase refunds and explicit funding
  reassignments/return-to-unassigned are settled in AC-5.20–5.21, including
  warning-only negative availability from over-reassignment. Corrected
  purchase-portion goal links are settled in AC-5.22: reverse the old goal's
  usage and apply it to the new goal as an auditable correction, correcting the
  category if needed. Remaining questions are limited to exceptional
  saved-reduction refunds, refunds/FX differences that exceed remaining linked
  usage, and allocating refunds across portions.
  Purchase portions spanning goals are settled in AC-5.22; allocating refunds
  across portions remains open. How is an exceptional saved-reduction
  transaction below used exposed? Target validation and exact monthly saving calculations also need
  definition; none of these changes can create a second expense.
- **Closed-budget corrections:** What is the fallback when current-period
  application is impossible? Which budget receives it when several remain
  open, and how are carryover consequences explained without rewriting closes?
- **Remaining multicurrency calculations:** Eligible foreign held assets
  automatically revalue budget funding in the MVP (AC-5.10); automatic versus
  reviewed user choice is later. Foreign-debt valuation and warning-only
  budget coverage are settled in AC-5.17. Budget assignments and goal targets
  themselves remain fixed CLP. Derived CLP foreign-asset valuations multiply
  the exact balance by the exact rate and round once to a whole peso, nearest
  with midpoint away from zero. Ordinary split
  rounding and visible residual distribution are settled (AC-3.12); refine the
  edge case where a source total has more precision than its normal allocation
  unit, preserving its exact supplied amount. Define significant-FX-change
  alert thresholds while preserving warning-only floating expense estimates.
  Account balances, actual conversion chains and reviewed aggregate-allocation
  defaults remain settled behavior.
- **Later-release FX range calculation:** What rate source/method and
  historical lookback settings should supply scenario ranges, and how should
  historical extrema be distinguished from trend-informed future estimates?
  The range view is later scope; display alongside fixed progress is accepted
  (FX-R1), without directly revaluing assigned goal funding or guaranteeing
  bounds. Eligible foreign-asset budget revaluation is separately specified
  in AC-5.10.
- **Pool resources:** How does a manual planning amount affect fallback
  relative to holdings, and how do forecast income/savings acquire pool
  locations? Known dated obligations reduce projected pool balances when they
  occur, not upfront. These signals cannot replace shared-category
  affordability or create goal funding.
- **Forecast edges:** How do schedules handle nonexistent dates and rule edits
  affecting overridden/linked occurrences? What additional actual-to-
  occurrence link cases are supported? The user explicitly decides whether a
  scheduled goal purchase and recurring/history estimate are the same cost or
  separate costs (AC-6.6); until marked as the same, count both. Do not infer
  this overlap silently. If marked as the same cost but the estimates differ,
  which amount drives the projection remains open; preserve both source amounts
  and do not imply an amount-selection rule.

The broader CLI write/review workflow, additional report comparisons and later
shared-expense semantics remain open in the product specification. They are
not silently resolved by this conceptual model.
