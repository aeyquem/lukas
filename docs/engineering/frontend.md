# Frontend engineering

Status: Accepted

The frontend app lives in `frontend/app` and uses Vite, Vue 3 and TypeScript.
It must run directly in a browser against the .NET API without Tauri.

## Application state

- Use Vue Router for durable workflows.
- Use Pinia for durable client-side UI/application state.
- Use Pinia Colada for server-state fetching, caching, invalidation and
  mutation coordination.
- Do not use Pinia as a mirror database for API data.

Generate TypeScript API DTOs from ASP.NET/OpenAPI into
`frontend/app/src/api/generated/`. Generated files are committed and should not
be edited by hand. Handwritten API clients, error mapping and composables live
under `frontend/app/src/api/`.

## UI

Use PrimeVue as the implementation source for reusable UI primitives through
project-owned Base components. Feature code should import local `Base*`
components instead of importing PrimeVue components directly, unless the
PrimeVue component is explicitly too complex or table-specific to abstract in
the current change. Base components should own repeated labels, descriptions,
validation messages, disabled/loading states, ARIA wiring and design tokens
without hiding domain behavior or data fetching.

Use CSS or SCSS with small project-owned utilities, composition classes,
component block styles and explicit state hooks such as `data-*` attributes.

Use Spanish as the only initial UI language. Centralize date, money, decimal and
sign formatting in frontend helpers/composables. Formatting must only present or
mask values; it must not change stored values or calculation inputs.

## Component tooling

Use Vitest and Vue Test Utils for unit/component tests. Use Histoire for base
components, design-system components and important feature states.
