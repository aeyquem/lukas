# Frontend conventions

These conventions describe the frontend patterns used in this repository. Keep
new code aligned with them; update this document when an accepted pattern
changes.

Use kebab-case for frontend filenames, including Vue components and views.
Keep conventional project documents such as `ARCHITECTURE.md`, `CONVENTIONS.md`,
and `README.md`, plus the `OFL.txt` license file, as exceptions. Component
symbols may remain PascalCase in TypeScript and templates.

Follow the [official Vue Style Guide](https://vuejs.org/style-guide/) for Vue
code. A specific file may override a convention when that file clearly states
the exception and its reason. This document's kebab-case filenames are the
project-wide choice; use PascalCase component symbols and tags in SFC code.

## Organize by feature

- Put feature-specific API contracts, queries, mutations, components and
  composables under `app/src/features/<feature>/`.
- Keep route views in `app/src/views/`. A view composes feature UI and owns
  route-level coordination such as selecting an entity by ID; it should not
  become the home for every form and request workflow on the page.
- Put cross-feature transport, pending financial commands and formatting in
  `app/src/shared/`. Keep shared code small and specific to a real shared need.
- Expose supported feature capabilities through a clearly named feature module,
  such as `accounts-feature.ts`; avoid a generic `index.ts` barrel. Consumers
  outside the feature use this public module. Code inside the feature imports
  its own API modules directly. Consumers should not reach into another
  feature's component internals.
- Name Pinia store modules with the `-store.ts` suffix, for example
  `theme-store.ts`. Name composable modules with the `use-` prefix, for example
  `use-dashboard-period.ts`.
- Name route SFCs with the `-view.vue` suffix and use `*.test.ts` for Vitest
  files. Keep exported Vue component symbols and TypeScript types PascalCase;
  use camelCase for variables, functions and composable exports.
- Keep account contracts and query/mutation definitions in `features/accounts`;
  transaction contracts and transaction queries belong in
  `features/transactions`.

## Keep state with its owner

- Use Pinia for client state genuinely shared across app features, currently
  appearance preferences. Do not copy query results into a Pinia store.
- Keep drafts, validation messages, pending form actions and captured baseline
  revisions local to the form or workflow that owns them.
- Treat fetched records as read-only snapshots. A refetch may update readback,
  but must not overwrite a dirty draft or advance the draft's expected revision.
- Keep shareable filters and periods in the URL. Keep transient UI state local.
- Derive display values with `computed`; use watchers for specific side effects
  such as initializing a draft when its entity or intent changes.

## Use Colada for server snapshots

- Define feature-owned Colada query keys next to their query functions. Include
  every input that changes the result, such as entity ID, page size and cursor.
- Disable queries when required identifiers are absent or invalid.
- Set freshness and refetch behavior explicitly. An explicit Refresh action
  should make a network read.
- Guard asynchronous readers when parameters change so late responses cannot
  replace the current selection's data. Reset paginated state when its entity
  changes.
- Show read errors alongside any retained cached data. Never turn an unavailable
  financial read into a fabricated zero or a confirmed value.

## Protect financial writes

- Keep writes behind the shared financial-command layer. Persist the exact
  request, idempotency key and installation generation before sending; fail
  closed if persistence fails.
- Wrap that layer in focused feature mutations. Do not add automatic retries or
  optimistic updates to authoritative financial values.
- Retry only by explicitly resending the stored command after checking its
  installation generation. Dismissing a notice must not delete an unresolved
  command.
- Keep a successful write receipt distinct from its follow-up read. A read
  failure after a confirmed receipt must not make the write appear retryable;
  display confirmed money only from an authoritative read.
- Invalidate the relevant query keys after confirmed writes. While a reader is
  still manually loaded, use the temporary finance invalidation bridge and
  retain that reader's request guards and draft ownership. Do not store query
  snapshots in the bridge.

## Make component boundaries clear

- Use Vue 3 Composition API with `<script setup lang="ts">` for components.
- Pass data down with typed props and report outcomes up with typed emits. Use
  `v-model` only when the child genuinely owns a two-way editing contract.
- Extract a component or composable when it owns a distinct UI responsibility,
  workflow, state lifetime or cleanup. Avoid catch-all page controllers and
  components that only forward unrelated props and events.
- Keep form state and its validation feedback with the form. Put pure
  calculations and draft rules in plain functions rather than stateful
  composables.
- Keep DOM behavior and lifecycle cleanup with the component or composable
  that owns it.

## TypeScript, templates and styles

- Keep TypeScript strict and public component/API contracts explicit. Prefer
  domain-specific types over `unknown` or broad page-controller objects where
  the shape is known.
- Keep SFC sections in this order: script, template, style. Use semantic HTML,
  native form controls and associated labels and error text.
- Keep scoped styles with their owning component. Use the existing CSS custom
  properties for theme colors, spacing and geometry; do not duplicate theme
  values in component styles.
- Preserve the numeric font on decimal and date inputs when a local input rule
  sets a broader `font` shorthand.

## Verification

- Add focused tests for pure draft rules, query keys and request behavior,
  mutation outcomes, conflict handling and state that must survive refetches.
- Use the existing browser-check scripts for behavior that depends on routing,
  layout, real API readback, exact retries or browser storage.
- Run the frontend unit tests, type check and production build after frontend
  changes. Run the relevant browser checks against a disposable development
  database when a check writes financial records.
- Keep browser-test fixtures clearly named and isolated. A browser check is
  useful only when it waits for the current route/document and exercises the
  current UI selectors and pagination behavior.
