# Implementation details to discuss

This document is for concrete programming and tooling decisions. It should stay
separate from `technical-details.md`, which captures product architecture,
domain boundaries and higher-level technical decisions.

Implementation details belong here when they answer questions such as:

- Which .NET version and project templates to use.
- How to package the C# sidecar with Tauri.
- Which test frameworks to use.
- Which Vue setup and frontend tooling to use.
- How command/query DTOs are shared or generated.
- Which ORM, migration tool or SQL approach to use.
- How the local development workflow is run.
- Which formatting, linting, type-checking and CI commands exist.

## Current implementation direction

These are not final implementation decisions yet, but they follow from the
current technical direction:

- Desktop shell: Tauri.
- Frontend: Vue 3.
- Styling: CSS or SCSS, without Tailwind-like utility CSS.
- Backend/domain: C#/.NET sidecar.
- Local database: SQLite.
- Future cloud backend: ASP.NET API with PostgreSQL.

## Topics to decide

### Repository layout

- Monorepo structure for desktop app, Vue frontend, C# sidecar, domain projects,
  storage projects, importers and tests.
- Naming conventions for projects and modules.
- Where shared command/query contracts live.

### Desktop packaging

- .NET version.
- Self-contained vs framework-dependent sidecar packaging.
- How Tauri starts, stops and monitors the sidecar.
- How local ports, sockets or IPC are selected.
- How logs are collected locally.

### Backend implementation

- ASP.NET host shape for the local sidecar.
- Endpoint style for command/query API.
- Validation library and error response format.
- ORM/query strategy for SQLite now and PostgreSQL later.
- Migration tooling.

### Frontend implementation

- Vite Vue vs another Vue setup for the desktop app.
- State management library or local composable pattern.
- Form validation approach.
- Data visualization library.
- Component organization and design-system conventions.

### Testing

- .NET unit test framework.
- .NET assertion and fixture libraries.
- SQLite integration test strategy.
- Importer test fixtures for CSV and Excel.
- Frontend unit/component test setup.
- End-to-end test setup for critical flows.
- Golden/snapshot tests for report and import-review outputs.

### Development workflow

- Local dev command for Tauri, Vue and sidecar together.
- Formatting commands.
- Linting and type-checking commands.
- Test commands.
- CI pipeline once implementation begins.
