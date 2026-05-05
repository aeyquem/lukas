# Development workflow

Status: Accepted

Run commands from the repository root unless noted.

## Main commands

```text
pnpm install
pnpm dev
pnpm dev:desktop
pnpm check
pnpm test
pnpm test:e2e
pnpm format
pnpm api:check
```

`pnpm dev` runs the ASP.NET API and Vite frontend together. `pnpm dev:desktop`
runs the API and Tauri desktop shell together.

## Tooling

- Use pnpm for JavaScript package and workspace management.
- Use Node as the frontend runtime baseline.
- Pin the .NET SDK with `global.json`.
- Pin Node and pnpm through `mise.toml`.
- Use Prettier and ESLint for frontend formatting/linting.
- Use `vue-tsc` for Vue/TypeScript checks.
- Use `dotnet format`, `dotnet build` and `dotnet test` for backend checks.
- Keep generated OpenAPI TypeScript files current with `pnpm api:check`.

Commit lockfiles, EF Core migrations, generated OpenAPI TypeScript files,
golden/snapshot expected outputs, Histoire stories and Playwright specs. Do not
commit `node_modules`, build output, local SQLite databases, logs, `.env.local`
or local AI/tooling state.

