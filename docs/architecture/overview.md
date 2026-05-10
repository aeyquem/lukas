# Architecture overview

Status: Accepted

Lukas starts as a local-first desktop application for a personal or
family-scale workspace. The first implementation is a modular monolith with a
Vue frontend, a thin Tauri shell, a C#/.NET sidecar API and SQLite persistence.

The financial domain, import logic, application services and calculations must
stay independent from Tauri, Vue, SQLite-specific behavior and any future cloud
host.

## Current shape

- Desktop shell: Tauri.
- Frontend: Vue 3 and TypeScript.
- Styling: CSS or SCSS, without Tailwind-like utility CSS.
- Backend/domain: C#/.NET sidecar.
- Local database: SQLite.
- API boundary: local REST/RPC-style command/query HTTP API.
- Initial import sources: manual entry, CSV, Excel and mock Open Finance.
- Future cloud backend: ASP.NET API, PostgreSQL and object storage.

## Call flow

```text
Vue UI
  -> typed frontend client
    -> local HTTP API exposed by the C# sidecar
      -> application services
        -> domain model
          -> persistence interfaces
            -> SQLite
```

Writes should be explicit commands because financial changes need traceability.
Reads should use query endpoints and screen-specific DTOs. The frontend must not
access SQLite directly.

## Frontend folders

The Vue frontend uses a feature-first folder architecture. Product workflows
live under `frontend/app/src/features/<feature>/` with their own components,
composables, models, stores, views, tests and stories as needed.

Shared code lives under `frontend/app/src/common/` only when it has no product
feature ownership or has proven reuse across features. Common code may not
import from feature folders. Feature-to-feature imports should stay rare; when
two features need the same stable concept, extract the smallest useful piece to
`common`.

Application bootstrapping, route setup and shell-level views live under
`frontend/app/src/app/`.

## Future host direction

A future web or SaaS version should reuse the same domain and application
services. The hard parts of that migration are expected to be auth, workspace
membership, encryption, sync, background jobs, operational visibility, provider
secrets and storage migration rather than the UI framework.
