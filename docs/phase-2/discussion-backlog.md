# Discussion backlog

This backlog captures items that need more discussion before they are added to
the v1 use-case specification. Items are grouped by section and identified as
`Sx.Ix` so they can be referenced quickly.

**1 Currencies / Conversion**

- [x] S1.I1 Different cards or banks may use different exchange rates for the
  same currency, date and transaction type.
  Why it matters: the app cannot assume a single daily FX rate. Reports,
  projections and reconciliations need to know which institution produced the
  final converted value.
- [x] S1.I2 A transaction can have several currency amounts: merchant amount,
  authorization amount, billed amount, paid amount, fees, taxes and final charge.
  Why it matters: pending and settled values can differ. The model needs enough
  structure to explain what the user saw at purchase time, what was billed and
  what finally affected the account.
- [x] S1.I3 Pending international transactions should use the best available
  institutional value until the final value exists.
  Why it matters: users need useful projections before settlement, but the app
  must make clear that pending converted amounts can still change.
- [x] S1.I4 Refunds may use a different exchange rate than the original purchase;
  once final values exist, those should be used.
  Why it matters: refunds may not cancel the original purchase exactly in CLP.
  Reports need to show the real net effect instead of assuming symmetric FX.
- [x] S1.I5 Card networks, banks, Wise, PayPal, brokers and manual rates may all
  disagree, so conversion source must be explicit.
  Why it matters: disagreements are expected. Storing the source avoids confusing
  corrections later and lets the user understand why two similar operations used
  different rates.
- [x] S1.I6 Some assets can be priced in one currency but reported in another.
  Example to discuss: a Chilean broker may show a portfolio balance in CLP while
  the underlying ETF, stock or fund is priced in USD.
  Why it matters: this affects net worth and investment reporting. We need to
  decide whether the app tracks the reported balance only, the underlying asset
  currency or both.
- [x] S1.I7 Historical reports should use the conversion rate stored for the
  transaction when available; historical FX lookup is only needed when the stored
  rate is missing.
  Why it matters: recalculating history with today's rate or a generic historic
  rate can change past reports. Stored rates preserve what actually happened.

**2 Credit Cards / Debt**

- [x] S2.I1 Credit card transactions need separate dates for purchase,
  authorization, statement/billing, due date and payment.
  Why it matters: the same charge can affect spending, statement balance and
  cashflow in different periods. This drives budget rollover, reports and debt
  projections.
- [x] S2.I2 Installment purchases should be supported as one purchase that
  creates multiple future charges, without trying to calculate interest rates.
  Why it matters: installments are common and affect future cashflow. We need a
  user-facing model that schedules charges without pretending to be a credit
  interest calculator.
- [x] S2.I3 Interest, late fees, maintenance fees and insurance can be treated as
  card ownership cost when the source does not make the detail clear.
  Why it matters: these charges should not disappear, but ambiguous statements
  should not force complex categorization. A default ownership-cost bucket keeps
  reports honest.
- [x] S2.I4 Partial credit card payments can be represented at a basic level
  through balance changes.
  Why it matters: users may pay less than the statement balance. The app should
  reflect remaining debt without needing full interest modeling in v1.
- [x] S2.I5 Overpayment on a card creates positive balance and should be treated
  as a valid state.
  Why it matters: a positive card balance is not an error. Account balance logic
  and reports need to allow it without forcing reconciliation noise.
- [x] S2.I6 Card statement corrections can happen after a budget period or
  statement was already closed.
  Why it matters: closed reports may need explicit corrections instead of silent
  rewrites. This affects auditability and period locking.

**3 Transfers / Internal Movement**

- [x] S3.I1 Transfers between own accounts should not affect the budget result;
  they can appear as an outflow in one account and inflow in another but net out.
  Why it matters: otherwise moving money between accounts looks like income or
  spending. Reports need to distinguish cash movement from financial result.
- [x] S3.I2 Cross-currency transfers need source amount, destination amount and
  conversion/spread handling. If the institution does not provide it, the user
  can enter fees/spread or the system can use a mid-market estimate.
  Why it matters: the user needs to see both sides of the transfer and any loss
  to spread or fees. This also affects FX reporting and reconciliation.
- [x] S3.I3 Transfers can appear as two imported transactions from different
  institutions and need matching.
  Why it matters: importing both accounts can create duplicate-looking movement.
  Matching avoids double counting while preserving each account ledger.
- [x] S3.I4 Moving money into savings, goals or emergency reserves is allocation,
  not spending, and should not distort the budget result.
  Why it matters: saving is a decision about purpose, not consumption. The app
  needs to report allocation separately from actual expenses.

**4 Cash / Manual Accounts**

- [x] S4.I1 Cash withdrawals should count as transfers to a cash account, not as
  expenses by default.
  Why it matters: the expense happens when cash is spent, not necessarily when it
  leaves the bank account. This keeps spending reports from overstating cash use.
- [x] S4.I2 Cash reconciliation differences can be unknown loss, forgotten
  expense, gift, rounding or another reason chosen by the user with a comment.
  Why it matters: cash discrepancies are often ambiguous. The system should let
  the user explain the adjustment instead of enforcing a single meaning.
- [x] S4.I3 Manual accounts can go negative. The system should warn and help the
  user get back to zero, while allowing valid negative states such as credit
  lines.
  Why it matters: negative balances can signal missing data or a real debt
  facility. The app should surface the risk without blocking valid scenarios.

**5 Imports / Integrations**

- [x] S5.I1 The same transaction can be imported from bank, card, Splitwise, CSV
  and Open Finance.
  Why it matters: multiple sources are a core feature. The app needs merge and
  source-tracking behavior before imported data becomes reliable.
- [x] S5.I2 External transaction IDs may change between syncs or may not exist;
  the user should be able to decide how to handle ambiguous cases.
  Why it matters: provider IDs are not always stable. Internal identity and user
  decisions are needed to prevent duplicates or accidental rewrites.
- [x] S5.I3 Institutions may mutate historical transactions after settlement;
  the user should decide whether to update, ignore or keep both values.
  Why it matters: banks can replace pending data with settled data or fix old
  records. The app must decide how much history can change after the user saw it.
- [x] S5.I4 Duplicate detection needs confidence levels and a "never suggest
  again" decision.
  Why it matters: false positives are costly. Confidence and dismissal rules make
  deduplication useful without repeatedly bothering the user.
- [x] S5.I5 Deleted or hidden transactions from a source may need tombstones
  instead of hard deletes.
  Why it matters: if the source stops sending a transaction, the app needs to
  know whether it was removed, hidden, dismissed or should be recreated later.

**6 Budget Periods**

- [x] S6.I1 Period reset rules can collide with weekends or holidays.
  Why it matters: real income and bills shift around non-business days. Budget
  periods need predictable rules so reports do not jump unexpectedly.
- [x] S6.I2 Salary or recurring income may arrive early because of weekends or
  holidays; rules per payer should allow assigning it to the intended period.
  Why it matters: early payment should not make the previous period look richer
  or the next one look short. This affects period-vs-projection reports.
- [x] S6.I3 A transaction can belong to different reporting periods at the same
  time. Example to discuss: a purchase on January 28 is in calendar January, may
  belong to the February card statement and may be assigned to the February
  budget period.
  Why it matters: one date is not enough for every report. The model may need
  separate calendar, statement and budget period assignments.
- [x] S6.I4 Closed periods may need locking, with explicit adjustments afterward.
  Why it matters: users need stable historical reports, but corrections still
  happen. Locking requires a clear adjustment workflow.

**7 Goals / Envelopes / Reserves**

- [x] S7.I1 One account funding multiple envelopes can accidentally double count
  money if allocation rules are not explicit.
  Why it matters: the same peso cannot fully fund two goals at once. This affects
  progress, emergency coverage and available-money reports.
- [x] S7.I2 A goal may be funded by accounts in different currencies; the goal
  can use either the base currency or a goal-specific currency.
  Why it matters: goal progress depends on conversion rules. We need to decide
  where FX risk is shown and which currency defines the target.
- [x] S7.I3 Emergency fallback rules need priority when multiple reserves can be
  used.
  Why it matters: in a shortage, the app must know which reserve is consumed
  first. Otherwise survival projections and goal impacts become ambiguous.
- [x] S7.I4 Goals can be paused, cancelled, missed, partially spent or overspent.
  Why it matters: goals are not always completed cleanly. The app needs states
  that reflect real life without corrupting history or projections.
- [x] S7.I5 A planned purchase can become a real transaction and must consume the
  reserved amount.
  Why it matters: once the purchase happens, the reserve should move from plan to
  actual spending. This prevents the same money from staying available.

**8 Survival Scenarios**

- [x] S8.I1 Essential spending may change under stress and should be adjustable
  by the user during the scenario.
  Why it matters: survival budgets are not just historical averages. The user
  needs to define what they would actually cut or keep.
- [x] S8.I2 Liquidity delays, penalties or taxes are assumptions until they are
  represented in the ledger.
  Why it matters: not all money is equally available. The scenario should expose
  assumptions without pretending they are confirmed transactions.
- [x] S8.I3 Investment values can change; institution-reported values should be
  preferred when available, otherwise a market reconciliation approach may be
  needed.
  Why it matters: investment balances are volatile. The app needs a way to show
  current value without treating market movement as normal spending.
- [x] S8.I4 Retirement-like accounts should distinguish non-withdrawable funds
  such as AFP from potentially withdrawable products such as APV.
  Why it matters: including non-withdrawable money in runway can give a false
  sense of safety. Account liquidity needs to be explicit.
- [x] S8.I5 Debt minimum payments may continue even in survival mode.
  Why it matters: reducing discretionary spending does not remove debt
  obligations. Survival calculations need to include required payments.
- [x] S8.I6 A survival scenario can include estimated severance or termination
  compensation as a projected inflow.
  Why it matters: a job-loss scenario is incomplete if it removes salary but
  ignores likely one-time exit payments. The estimate must be visible as an
  assumption, not treated as confirmed income or legal advice.

**9 Reports**

- [x] S9.I1 Financial health should avoid false precision. Example to discuss:
  instead of a single exact score, reports can show rule status, confidence and
  which assumptions were used.
  Why it matters: a single score can hide uncertainty and user context. Showing
  rules and assumptions makes the report more trustworthy.
- [x] S9.I2 Reports need to distinguish actual, pending, projected, adjusted and
  reconciled values.
  Why it matters: mixing these values makes reports hard to trust. Users need to
  know what is final, estimated or manually corrected.
- [x] S9.I3 Spending reports should handle refunds and reimbursements cleanly.
  Why it matters: gross spending and net spending answer different questions.
  Refunds and reimbursements should not distort category analysis.
- [x] S9.I4 Shared expenses should show gross spend, user share, pending
  reimbursement and final net spend.
  Why it matters: shared payments can look much larger than the user's real cost.
  Reports should separate what was paid from what was owed.
- [x] S9.I5 Net worth can stay simple for now and use available information, but
  may later separate liquid, illiquid, restricted and volatile assets.
  Why it matters: simple net worth is useful early, but liquidity and risk matter
  for decision-making. We should avoid a model that blocks that later split.

**10 Categories / Tags**

- [x] S10.I1 Transactions should support categories and subcategories.
  Why it matters: high-level categories are too broad for useful spending
  analysis. Subcategories let reports answer more specific questions.
- [x] S10.I2 Transactions should support custom labels/tags independent from
  category.
  Why it matters: tags capture temporary or cross-cutting context such as trips,
  projects, people or tax review without changing the category tree.
- [x] S10.I3 Reports should support filtering and grouping by category,
  subcategory and tag.
  Why it matters: categorization only becomes useful when reports can slice by
  those dimensions.
- [x] S10.I4 User edits to category, subcategory or tags should be preserved even
  when the transaction is imported again.
  Why it matters: imports should not overwrite user intent. Preserving edits
  keeps automation from undoing cleanup work.
