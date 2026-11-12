# Proposal

## Why

The budget page exposes useful calculations but spreads them across summary cards and a flat table, making it harder to understand available money and organize categories while assigning it. A focused workspace modeled on familiar envelope-budget layouts can make the monthly plan easier to scan and maintain without changing Lukas's financial calculations.

## What Changes

- Replace the scattered summary presentation with a prominent “Available to budget” card that explains its calculation using the available authoritative breakdown, including income and carried unassigned funds where available.
- Show category-to-category rollover as zero in the available-to-budget breakdown; do not count those balances as available money. Explain actual signed category remainders released by the already-supported explicit close choice separately.
- Keep exactly one table in the Presupuesto tab, titled “Presupuesto” instead of “Categorías”. Move “Administrar categorías” into a separate tab; preserve its capabilities and defer further administration design refinements.
- Organize budget categories into named groups/themes, with useful predefined groups and owner-managed add, rename, reorder, and archive/remove behavior.
- Provide an edit mode that enables rearranging groups and categories; keep arrangement controls locked outside that mode.
- Allow assignment amounts to be edited directly in the table, retaining authoritative save and stale-revision behavior.
- Move further category editing into a right-side detail drawer. Archive categories to remove them from active budgeting while retaining historical references; permanently delete only categories with no transactions.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `monthly-budget`: specify the explained available-to-budget presentation, grouped and reorderable categories, inline assignment editing, and category archive/deletion behavior.

## Impact

- Frontend budget view, category management, assignment editing, and category detail drawer.
- Budget and category API contracts and PostgreSQL persistence for group membership/order and group management.
- Existing `monthly-budget` spec. Existing financial calculations, period close rules, decimal-safe values, revisions, and historical snapshots remain authoritative.
