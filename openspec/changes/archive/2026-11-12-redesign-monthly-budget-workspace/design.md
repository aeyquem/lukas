# Design

## Context

See proposal.md for motivation and `specs/monthly-budget/spec.md` for the observable contract. The current budget snapshot has totals for funding, ready-to-assign, activity, uncategorized activity, and flat categories. The category API and `expense_categories` table have a global sort order, but no grouping. Assignment drafts already use local component state and authoritative revisions.

The accepted calculation is server-side: ready-to-assign carries the prior unassigned amount, adds distinct period funding effects and any category remainder explicitly released at close, then subtracts assignments. Ordinary category remainder rolls forward within its category. Expenses affect category activity, not ready-to-assign a second time. Incomplete inputs must remain incomplete.

## Goals / Non-Goals

**Goals:**

- Make available-to-budget the primary summary and explain it with authoritative components.
- Add persistent, owner-managed budget groups and stable order within and between groups.
- Make rearrangement an explicit edit-mode action.
- Keep assignments revision-safe while editing the assigned value in the category row.
- Move category detail editing and archive/delete actions into a right drawer.

**Non-Goals:**

- Change funding, rollover, close, transaction-date, or closed-snapshot rules.
- Change the existing explicit close choices: preserve both category carry and release, explaining actual signed releases without adding ordinary category carry to available-to-budget.
- Refine the separate category administration tab beyond relocating its existing capabilities and the group/drawer support required here.
- Add income-category budgeting, automatic budget templates, or spreadsheet paste/formulas.
- Add a general-purpose drag-and-drop dependency unless native accessible controls prove insufficient.

## Decisions

### Keep calculation and explanation on the backend

Extend the budget read model with a typed available-to-budget breakdown derived from the same calculation inputs as the authoritative total. It should identify prior unassigned carry, distinct funding components (including income where the backend can distinguish it), explicit category-remainder releases, and assignments. Category-to-category carry is not an available-to-budget component and is shown as zero. Explicit category releases already supported at close are a separate signed component. The browser renders the returned component values and completeness/reasons; it does not calculate financial totals with JavaScript numbers.

If a component cannot be supplied or valued, expose that state instead of fabricating a zero. The current calculation already supports explicit signed category releases at close; return their actual amount, including zero when no releases apply. Keep ordinary category carry in the category's remaining calculation and show zero from that carry in the available-to-budget explanation.

### Persist groups as workspace budget data

Add a workspace-scoped group entity with name, archive state, and order. Add group membership and within-group order to expense categories while preserving category identity and transaction references. Seed a small set of suggested groups (General, Transport, Savings) once per workspace and associate existing starter categories deterministically during migration/bootstrap. Owners can create and rename groups; category and group reordering persists through explicit mutations.

Only active groups/categories appear in the normal budget table. A group can be archived after its active categories have been moved or archived, avoiding invisible active categories. Keep archived groups and categories available to historical records. Permanent category deletion is a separate guarded operation and must be rejected when transactions or retained budget history still reference it; archival remains available.

### Use one grouped table with an explicit arrangement mode

The Presupuesto tab contains exactly one table, titled “Presupuesto” (replacing “Categorías”). Move “Administrar categorías” into its own separate tab while preserving its capabilities; defer further administration refinements. Summary details and close controls must not add another table to the Presupuesto tab.

Render groups as table section headers and category rows beneath them, with assigned, used, and remaining values. The assigned value is an inline decimal-string input in the assigned cell. Preserve local drafts, server revision checks, pending-write recovery, and authoritative refresh behavior. Commit on Enter or an explicit keyboard-accessible confirmation within the cell; do not optimistically change summary totals. Closed periods retain their correction-reason requirement.

An “Edit” control enables drag handles and group/category arrangement actions. Outside this mode, rows and groups are stable and cannot be dragged. Provide keyboard-accessible move-up/down or move-to-group controls so rearrangement does not depend on pointer dragging. Save ordering through the category/group API and refresh the budget after success.

### Use a right-side category detail drawer

Opening a category's detail action presents a labeled, dismissible right-side drawer containing editable name/group fields and archive or eligible permanent-delete actions. Keep order changes in the arrangement mode. Preserve category-in-use protections and stale-write feedback. The drawer must work at narrow widths as a full-width panel and maintain keyboard focus when opened and closed.

### Keep changes additive and reversible

Use a database migration to add groups and category membership/order, backfill existing categories into seeded groups, and preserve existing category IDs and history. Deploy backend/API support before the frontend relies on group fields. Retain migrated data during application rollback: a compatible application build can ignore additive group fields, but its SQLx migrator must recognize the applied migration set. An older embedded binary is not assumed to start against a newer migration set. Legacy category INSERT compatibility is preserved and tested. Do not rewrite existing budget snapshots.

## Risks / Trade-offs

- [The total may be available while a component breakdown is incomplete] → Return component states and reasons independently; never imply that an unknown component is zero.
- [Group migration may produce a surprising initial layout] → Use stable documented mappings for seeded categories and allow immediate owner edits.
- [Hard deletion can conflict with retained assignment or close history] → Keep archive as the normal action and guard permanent deletion against every retained reference, including transactions and snapshots.
- [Drag interactions can exclude keyboard and touch users] → Include keyboard move controls and use native controls with visible focus states.
- [Inline saves can conflict with stale revisions] → Retain the draft on conflict and require explicit review/refetch, following existing budget write behavior.

