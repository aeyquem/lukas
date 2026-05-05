# Testing

Status: Accepted

Run `pnpm test` before opening a PR. Run `pnpm test:e2e` when changing routing,
startup or user-facing flows.

## Backend

- Use xUnit v3 for .NET tests.
- Use Shouldly for assertions.
- Use handwritten test data builders first.
- Use real SQLite databases for integration tests.
- Prefer temporary file SQLite databases when persistence, migrations or
  database lifecycle behavior matters.
- Keep CSV and Excel importer fixtures under
  `backend/tests/Lukas.Importers.Tests/Fixtures/`.

## Frontend

- Use Vitest and Vue Test Utils.
- Keep Vue `.spec.*` files next to the component, composable or module they
  test.
- Use Histoire stories for component states and visual review.

## E2E

Use Playwright for browser-mode end-to-end tests first. Add Tauri-specific E2E
coverage later for shell behavior such as sidecar startup, sidecar failure and
desktop packaging.

Financial correctness tests should focus on money, periods, imports,
deduplication, source history, transfers, projections and report stability.

