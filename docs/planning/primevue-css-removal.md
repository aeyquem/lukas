# PrimeVue CSS Removal Handoff

Status: Superseded by the project variables layer

## Supersession Note

The cleanup below described the previous direction: keep only PrimeVue Aura
tokens active and avoid project-owned global tokens. The current direction is to
keep PrimeVue Aura as the visual baseline while adding a small project CSS layer
in `frontend/app/src/styles/variables.css`.

The project variables use the `--lu-*` prefix and map back to PrimeVue semantic
tokens for color, radius and surface behavior. Feature and layout CSS should use
the project variables so spacing, typography, shadows and app-specific surfaces
remain consistent as the design system grows.

## Goal

Remove the active custom design-system CSS from the frontend so PrimeVue Aura is
the styling baseline. Future visual customization should use PrimeVue theming
instead of project-owned global color, spacing, font, radius, shadow, or motion
tokens.

This pass was intentionally tokens first. It did not introduce a new PrimeVue
preset, redesign the application, or remove domain-state styling that PrimeVue
cannot infer from the data.

## Current Wiring

- `frontend/app/src/main.ts` imports `frontend/app/src/styles/main.css`.
- `frontend/app/histoire.setup.ts` imports the same stylesheet for stories.
- `frontend/app/src/primevue.ts` installs PrimeVue with the Aura preset from
  `@primeuix/themes/aura`.
- PrimeVue and `@primeuix/themes` are already dependencies of
  `frontend/app/package.json`.

## Removal Strategy

- Keep `main.css` limited to browser-level necessities: box sizing, body margin,
  inherited form fonts, disabled cursor, base heading/paragraph margin reset,
  `.app-shell`, and reduced-motion handling.
- Remove the Google font import and every project `:root` design-system token
  from active CSS.
- Replace old `var(--color-*)` references with PrimeVue semantic variables such
  as `--p-text-color`, `--p-text-muted-color`, `--p-content-background`,
  `--p-content-border-color`, `--p-content-hover-background`,
  `--p-primary-color`, and `--p-primary-contrast-color`.
- Replace old spacing, radius, font-size, shadow, and motion tokens with local
  CSS literals only where component layout still needs them.
- Prefer removing PrimeVue structural overrides over translating them. Keep
  selectors like `:deep(.p-datatable-thead > tr > th)` only when they preserve
  behavior or readable layout.
- Keep domain-state CSS when it communicates application data, for example
  uncategorized movement rows, budget drag state, negative amounts, and warnings.
- Prefer `BaseButton` over local hand-styled buttons. Raw buttons should remain
  only when the control is not a normal command button, such as sidebar
  navigation or the budget assigned-amount inline edit affordance.

## Affected Areas

- Global stylesheet: `frontend/app/src/styles/main.css`.
- App shell and layout: `HomeView.vue`, `AppHeader.vue`, `AppSidebar.vue`.
- Base-adjacent UI: `BaseField.vue` and base Histoire stories.
- Ledger and workspace surfaces: `AccountsPanel.vue`,
  `LedgerMovementsTable.vue`, `MovementsWorkspace.vue`,
  `BudgetWorkspace.vue`, `GoalsWorkspace.vue`, `ReportsWorkspace.vue`, and
  `ProjectionsWorkspace.vue`.

## Archived Design-System CSS

The following CSS was removed from the active app. It is kept here only as
historical reference in case a future PrimeVue theme wants to reuse values.

```css
@import url("https://fonts.googleapis.com/css2?family=DM+Serif+Display&family=JetBrains+Mono:wght@500;700&family=Plus+Jakarta+Sans:wght@400;500;600;700;800&display=swap");

:root {
  --color-background: #eef3eb;
  --color-foreground: #182b1a;
  --color-card: #ffffff;
  --color-primary: #2d6a4f;
  --color-primary-strong: #1b4332;
  --color-primary-foreground: #ffffff;
  --color-secondary: #dce8de;
  --color-secondary-foreground: #2d6a4f;
  --color-muted: #e4ede5;
  --color-muted-foreground: #5b7a60;
  --color-accent: #40916c;
  --color-accent-foreground: #ffffff;
  --color-border: rgb(45 106 79 / 0.13);
  --color-border-strong: rgb(45 106 79 / 0.24);
  --color-input-background: #f7faf6;
  --color-ring: #52b788;
  --color-success: #3a9e6e;
  --color-warning: #a8690b;
  --color-danger: #c0392b;
  --color-danger-strong: #9f2d22;
  --color-danger-foreground: #ffffff;
  --font-body: "Plus Jakarta Sans", Inter, ui-sans-serif, system-ui, sans-serif;
  --font-display: "DM Serif Display", Georgia, serif;
  --font-mono: "JetBrains Mono", ui-monospace, monospace;
  --font-size-xs: 0.75rem;
  --font-size-sm: 0.875rem;
  --font-size-md: 1rem;
  --space-1: 0.25rem;
  --space-2: 0.5rem;
  --space-3: 0.75rem;
  --space-4: 1rem;
  --space-5: 1.25rem;
  --space-6: 1.5rem;
  --space-8: 2rem;
  --radius-sm: 4px;
  --radius-md: 6px;
  --radius-lg: 8px;
  --radius-round: 999px;
  --shadow-sm: 0 12px 28px rgb(24 43 26 / 0.08);
  --shadow-lg: 0 24px 60px rgb(24 43 26 / 0.16);
  --shadow-focus: 0 0 0 4px rgb(82 183 136 / 0.18);
  --motion-fast: 180ms ease;
  color: var(--color-foreground);
  background: var(--color-background);
  font-family: var(--font-body);
}
```

## Validation

- Run `pnpm --dir frontend/app typecheck`.
- Run `pnpm --dir frontend/app lint`.
- Run `pnpm --dir frontend/app test`.
- If visual risk is high, run Histoire or the Vite app and inspect the account,
  movement, budget, goals, reports, and projections sections.
