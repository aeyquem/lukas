# Frontend architecture

Status: **first-slice direction selected and independently reviewed**. Future
authentication and cross-origin deployment details remain open. Use
`desc/13-first-slice-decisions.md` for the resolved first-slice API, routing,
interaction, accessibility, and deployment contracts; validate them during
implementation.

This document extracts the current frontend direction from `desc/09-technical-stack-proposal.md` and reviews it against the accepted rewrite context, first-slice plan, and backend architecture. It is a planning decision record, not a frozen implementation specification. Product behavior in `desc/05-mvp-acceptance.md` and `desc/08-first-slice-plan.md` takes precedence over UI convenience.

## Product and deployment fit

- Build a web client for the owner-only first instance. The owner may manage personal or household finances; separate member accounts are later scope.
- The client is a static Vue application, independently deployable in its own Docker image. It communicates with the independently deployable Rust backend, which uses PostgreSQL.
- Use Docker Compose with the local Docker Engine for development. Production deployment must not depend on Compose. Runtime route fallback is needed. The initial local deployment uses one canonical loopback origin and same-origin `/api` through the frontend container; cross-origin hosting and its proxy/TLS setup remain later deployment decisions.
- From the first working slice, exercise the real path from browser to Rust HTTP API to PostgreSQL and back. Develop the UI and API contract together for each vertical slice; avoid a frontend that only works against a permanent mock API.
- Local development uses Docker Compose with Vite hot reload inside a
  container. Bind source files into the Vite container, retain dependencies
  built for the container separately from host `node_modules`. Bind individual
  source paths so the image's container-built dependencies are not obscured;
  rebuild from the lockfile when its manifest or lockfile changes. If a
  persistent dependency volume is used instead, explicitly refresh it from the
  lockfile on dependency changes so it cannot mask new image dependencies.
  Native filesystem events
  are the default; enable polling only when the host filesystem does not pass
  events through, because it can use more CPU. See [Vite watcher options](https://vite.dev/config/server-options#server-watch)
  and [Docker bind mounts](https://docs.docker.com/engine/storage/bind-mounts/).
  This accepted development workflow does not change the production images or
  initial access boundary.
- The first version has no authentication and is limited to the same machine: publish the frontend on loopback, keep the backend port private to the container network, and send browser requests through same-origin `/api`. Do not expose this version on a LAN or through a public proxy. Wider access requires an explicit access-control design. Keep the boundary open for later account registration with email/password, OpenID Connect providers such as Google or Microsoft, or both; provider choice and future identity/session design remain undecided.
- Keep domain rules and authoritative calculations on the backend. The client supports review and data entry but does not make financial facts authoritative.

## Stack and boundaries

| Area | Current direction | Status |
| --- | --- | --- |
| Framework/build | Vue 3 Composition API, strict TypeScript, Vite, Vue SFCs | Accepted starting direction |
| Rendering/deployment | Client-rendered static assets in a separate frontend image, with runtime API configuration and `/api` reverse proxy | Selected for the first slice; validate during implementation |
| Routing | Vue Router; feature route, filters, report period and period basis live in shareable URLs | Accepted direction |
| Styling | CSS custom properties and cascade layers; scoped SFC CSS; native semantic HTML; small app-owned components | Accepted direction; existing `explore/` material is a seed, not approved branding or a finished system |
| Forms | Native controls, local Vue form state, browser constraints and small workflow validators | Accepted direction; backend validation remains authoritative |
| API access | Native `fetch` behind one small typed client; centralize base URL, credentials, serialization, structured errors and request IDs | Selected per use case; no generated client until serialization is proven |
| Server snapshots | Pinia Colada for query/cache behavior | Accepted direction; confirm version/configuration and mutation invalidation details during implementation planning |
| Tables/editors | Begin with semantic HTML tables and inputs; do not add a spreadsheet grid until a concrete workflow requires it | Selected; first-slice Enter/Escape and pagination details are in `desc/13-first-slice-decisions.md` |
| Charts | Graphics are required; compare Chart.js, ECharts or app-owned SVG when implementing the first chart | Package deliberately deferred |
| Tests/accessibility | Vitest, Vue Testing Library, Playwright; automated accessibility checks where practical and manual keyboard/assistive-technology review for critical workflows | First-slice manual review: keyboard-only and NVDA with Firefox; support Firefox/Chrome, smoke-check Zen, and use the list sizes in `desc/13-first-slice-decisions.md` |
| Locale | Default to `es-CL` and `America/Santiago`; explicit currency codes where `$` is ambiguous | Accepted direction; preserve date-only business dates and decimal strings |
| Connectivity | Online application that requires a reachable API; no offline mode | Accepted direction |
| Authentication | None in the initial version; leave room for later email/password registration and/or OIDC sign-in | Initial behavior accepted; future design open |

The first screen is an empty dashboard; onboarding is deferred. Use plain
semantic HTML and basic CSS. In a multi-field creation form, Enter submits the
complete item after validation and Escape discards the draft. In a grid cell,
Enter saves that field and Escape restores its previous value. Save failures
show an alert with details and Retry/Cancel choices. Use paginated transaction
lists. Keep field validation and accessible error feedback clear. Bulk paste
and spreadsheet behavior are not assumed for the first slice.

The selected local route map is `http://localhost:4173` → loopback `dev-proxy`
→ Vite on private `vite:5173`; browser `/api/...` requests pass through Vite to
`api:8080` with the `/api` prefix and original Origin preserved. HMR WebSocket
upgrades use the same host and proxy. Only the proxy is host-published; API,
Vite and PostgreSQL have no host ports. The proxy validates exact Host
`localhost:4173`; the API disables CORS and requires exact Origin
`http://localhost:4173` on browser writes. Do not use `allowedHosts: true`. SPA fallback serves
`index.html` only for client routes and never for `/api` requests. Verify route,
Host, Origin and WebSocket behavior during implementation. See [Vite server
options](https://vite.dev/config/server-options).

The selected production frontend image serves the built static assets with an
SPA route fallback and reverse-proxies `/api` to its configured backend
upstream. At startup it reads `API_UPSTREAM` for the proxy and generates a
non-secret `/runtime-config.js` from the public `API_BASE_URL`. Load the script
before the Vue app entry point; the same image can then target different
backend locations. The initial same-origin value is `/api`. Vite and Compose
are not required in production.

The selected UI contract stores only an unresolved command's exact request,
key and generation binding in origin storage before sending; storage failure
must stop the send. Unknown outcomes survive alert dismissal and reload and
can be retried explicitly with the exact request. A database-generation change
blocks old retries. Browser storage contains financial data and may be cleared;
it is not durable history. The details and restore caveat are in
`desc/10-backend-architecture.md` and `desc/13-first-slice-decisions.md`.

## Framework comparison experiment (optional decision evidence)

Vue is selected as the starting framework, so a Vue/Svelte comparison is not a blocker for beginning the integrated first slice. If performance or maintenance concerns make a comparison worthwhile, compare three implementations: Vue SFCs with the standard renderer, Vue Vapor mode, and Svelte 5. The user's hypothesis is that compiler-based rendering may improve runtime performance; test that hypothesis rather than assuming it decides the framework. Vue describes Vapor as a compilation mode targeting lower baseline bundle size and better performance, but it supports a subset of Vue APIs and mixed Vapor/virtual-DOM component-library interop has limitations. Svelte also compiles components. The relevant question is whether any measured difference matters for Lukas's workflows and whether the selected libraries and code remain easy to maintain.

### Experiment design

1. **Time box:** one or two focused days for a throwaway spike, then record the result. Do not build production features three times.
2. **Hold inputs constant:** use the same TypeScript level, fixture data, CSS, API stub, test cases, browser versions, machine and production build settings. Use equivalent routing and query-cache capabilities and count any extra adapter/library code each option needs.
3. **Implement the same small workflows:** a transaction form with date, decimal-string amount and field error; a semantic transaction table with search/filter; and an editable draft that survives a server refetch and retains edits after a stale-write response. Include the planned full-stack API call shape, even if the spike uses a deterministic local API stub.
4. **Measure:** production compressed initial and route-specific JS/CSS; time to usable screen; input-to-update latency on a small and larger fixed table fixture; memory and DOM/component count at the larger fixture; and keyboard/accessibility behavior. Run repeated production-build browser trials in Firefox and Chrome, report medians and spread, and smoke-check Zen. Keep backend/network time separate from client rendering time.
5. **Compare development cost:** have one developer implement the same follow-up change in each option (add a field, show a server validation error, and adjust the save response). Record elapsed time, code touched, type/tooling friction, package compatibility and any accessibility regression. This is a small structured review, not a claim of statistically significant productivity measurement.
6. **Decide by gates:** all variants must preserve the required behavior, strict typing and keyboard-accessible semantics. Prefer the simplest option that meets those gates. Treat speed as decisive only if repeatable gains show up in a user-visible interaction or meaningfully reduce shipped assets at realistic data sizes. If results are close, keep Vue as selected. Adopt Vapor only after confirming maturity and compatibility with Vue Router, Pinia Colada, component primitives and required tooling.

Vapor and Svelte share a compile-time rendering approach, but should not be assumed interchangeable: APIs, compiler constraints, component/library compatibility and developer workflow still differ. Vue's official documentation identifies page-load and update performance as distinct concerns and recommends measuring the app's actual use case. See [Vue performance guidance](https://vuejs.org/guide/best-practices/performance), [Svelte compiler documentation](https://svelte.dev/docs/svelte/svelte-compiler), and [Vue core's Vapor release notes](https://github.com/vuejs/core/releases).

## UI component strategy experiment (optional)

The starting choice is native HTML plus a small app-owned component layer. Before adopting a broad UI suite such as PrimeVue, compare it with the native approach in a small disposable screen. This is a choice about how much behavior and styling to own; it is separate from the Vue/Svelte rendering experiment.

Compare these variants within the selected Vue stack:

1. **Native:** semantic HTML controls, app-owned wrappers for shared labels/errors/buttons, and the existing CSS token/layer approach.
2. **PrimeVue styled:** PrimeVue controls with its theme/token customization configured to match the app's visual direction.
3. **PrimeVue unstyled:** PrimeVue interaction primitives styled with app-owned tokens, to test whether its behavior can be reused without making the product adopt its theme. PrimeVue documents both styled and unstyled modes; unstyled mode still requires application styling. See [PrimeVue theming modes](https://v4.primevue.org/theming/unstyled/) and [configuration](https://v4.primevue.org/configuration).

Build the same small screen in each variant: text/decimal amount input, date input, select, checkbox, button and disabled/loading/error states, a confirmation dialog, and a compact account or transaction table. Include labels, help text, field-level server errors, keyboard navigation, focus restoration, narrow layout and `es-CL` formatting. Keep money/date semantics and backend validation identical. Do not let a UI package perform financial validation or calculations.

Review each variant against the same checklist:

- time to implement the screen and the amount of app-specific wrapper/glue code;
- ability to apply app tokens, focus styles, density, responsive rules, and visual choices without fighting defaults;
- keyboard and screen-reader behavior in Firefox and Chrome, plus a Zen smoke check; validate the actual controls rather than relying on a library's accessibility claim;
- styling override complexity, global CSS/cascade behavior, and how easily a component can be replaced;
- production route asset increase and runtime behavior;
- effort to add a field-level backend error and a new state (loading, disabled, warning) consistently;
- integration with local draft ownership, `fetch`/Pinia Colada and the basic save/cancel flow.

Time-box this to a few hours or one focused day. Record which controls genuinely save work and which are already simple with HTML. The decision can be mixed: keep forms native while adopting one library primitive for a dialog or select if it clearly improves accessibility and keyboard behavior. If PrimeVue is selected, wrap used controls in small app-owned components and keep product tokens and behavior in the application so screens do not depend directly on one library's theme conventions. A data-grid product is a separate decision from PrimeVue's ordinary table component.

Adopt the general UI suite only if it reduces total implementation and maintenance effort while matching the app's visual tokens, accessible behavior, browser support and bundle budget. If results are close, keep native controls and add focused primitives only when a real workflow needs them.

## State ownership and financial trust

Keep these state types separate:

1. **Financial facts and calculations:** backend only.
2. **Fetched snapshots and request status:** Pinia Colada.
3. **Unsaved edits:** local component/composable drafts that retain their baseline revision.
4. **Ephemeral view state:** local selection, focus and expanded rows.
5. **Shareable view context:** URL filters, selected periods and report basis.

Never bind editable fields directly to cached server objects. A refetch must not erase a dirty draft. On a revision conflict, retain the draft and offer an explicit reload/review path. Mutations for consequential decisions are explicit user commands. Initially invalidate affected queries after a successful write; do not optimistically present derived money as confirmed. A timed-out write has an unknown outcome and must not be blindly replayed; use the accepted idempotency key and exact request. Record revisions and stale-update behavior remain to be agreed before those flows are implemented.

Represent API money and rates as decimal strings with currency codes; do not use JavaScript `Number` for authoritative arithmetic. Formatting and any chart coordinate conversion are display-only. Preserve `YYYY-MM-DD` business dates without viewer-timezone shifting, and keep known transaction time/offset separate. Do not invent a time where the source only provides a date. Client calculations or previews must be clearly provisional unless confirmed by the server.

Keep provenance and review evidence visible where relevant: original source values, later interpretations, user decisions, correction history, and effects on a closed-period snapshot. Label actual, pending, estimated, adjusted and reconciled values distinctly. Warnings such as over-assignment or uncovered card debt must not block valid actual activity where the acceptance criteria allow it.

## Organization and interaction conventions

Organize by product feature with a small shared layer:

- `app/` for bootstrapping and routing;
- `features/` for accounts, transactions, imports, budget, goals and reports;
- `shared/` for genuinely reused UI, formatting and API client code.

Keep feature-specific components and composables within their feature. Promote code to `shared/` only after reuse is real. Avoid generic abstraction layers and cross-feature implementation imports.

Use semantic roles, labels and names so workflows work with keyboard users, assistive technology, deterministic browser automation and AI-operated browsers. Dialogs need managed focus entry/return and Escape behavior, errors need accessible announcement, and motion must respect `prefers-reduced-motion`. Dense import and allocation workflows need a defined narrow-screen behavior. Charts need an exact-value accessible table or equivalent and a textual summary; charts are never the sole explanation.

## Decisions outside the first slice

- Future identity, account recovery, and session design; wider-than-loopback
  access control; and cross-origin frontend/backend hosting.
- Import bulk-selection and partial-failure behavior, split/allocation editing,
  and whether a spreadsheet editor is justified by a later concrete workflow.
- If a grid or general UI suite is justified later, compare accessibility,
  bundle cost, licensing, and self-hosted distribution needs. The first slice
  uses semantic tables and native controls.
- Chart questions, filters, accessible exact-value alternatives, and historical
  net-worth trends.

## References

- `desc/00-rewrite-context.md` — accepted rewrite and domain constraints.
- `desc/05-mvp-acceptance.md` — observable product behavior.
- `desc/08-first-slice-plan.md` — accepted product boundary for the first implementation slice.
- `desc/09-technical-stack-proposal.md` — broader stack and frontend review.
- `desc/10-backend-architecture.md` — backend/API contracts and security dependencies.
