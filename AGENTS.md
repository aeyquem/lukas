# Repository Guidelines

Never break any of the following rules on your own. if you need to add or modify something that breaks them ask for permission first and explain why you need to break them.

## Project Structure & Module Organization

This is a PNPM workspace for Lukas, a personal finance PoC. The Vue 3 frontend lives in `frontend/app`, with source under `frontend/app/src`, base components in `src/components/base`, router setup in `src/router`, and styles in `src/styles`. The ASP.NET backend is in `backend`, with production projects under `backend/src/Lukas.*` and xUnit tests under `backend/tests/Lukas.*.Tests`. The Tauri desktop shell is in `apps/desktop`, and Playwright end-to-end tests are in `tests/e2e/specs`. Planning and requirements documents live in `docs`.

## Build, Test, and Development Commands

Run commands from the repository root unless noted.

- `pnpm install`: install workspace dependencies.
- `pnpm dev`: run the ASP.NET API and Vite frontend together.
- `pnpm dev:desktop`: run the API and Tauri desktop app together.
- `pnpm check`: build the .NET solution, lint the frontend, and run Vue type checks.
- `pnpm test`: run backend xUnit tests and frontend Vitest tests.
- `pnpm test:e2e`: run Playwright specs from `tests/e2e`.
- `pnpm format`: apply `dotnet format` and Prettier formatting.
- `pnpm api:check`: regenerate OpenAPI TypeScript output and fail if generated files changed.

## Coding rules

- Restrict all code to very simple control flow constructs – do not use goto
statements, setjmp or longjmp constructs, and direct or indirect recursion.
- All loops must have a fixed upper-bound. It must be trivially possible for a
checking tool to prove statically that a preset upper-bound on the number of iterations
of a loop cannot be exceeded. If the loop-bound cannot be proven statically, the rule
is considered violated. This rule does not, of course, apply to iterations that are meant to be
non-terminating (e.g., in a process scheduler). In those special cases, the reverse rule
is applied: it should be statically provable that the iteration cannot terminate.
- No function should be longer than what can be printed on a single sheet of
paper in a standard reference format with one line per statement and one line per
declaration. Typically, this means no more than about 60 lines of code per function.
- Data objects must be declared at the smallest possible level of scope.
- All code must be compiled, from the first day of development, with all
compiler warnings enabled at the compiler’s most pedantic setting. All code must
compile with these setting without any warnings.
- Exceptions should be caught at the level they orignate and handeled accordingly using the Result/Maybe pattern for expected errors or crashing the program for critical errors.
- Avoid null/undefined and use Maybe for data that may or not exist. nulls are a liability and should not exist in the program.
- All edits should be done one layer at a time, and use tests to validate that the layer works before coding the next layer. the program might not compile at first but must do so, without warnings, when the last layer is added.

## Coding Style & Naming Conventions

C# uses nullable-enabled SDK-style projects with warnings treated as errors via `backend/Directory.Build.props`. Use four-space indentation, `PascalCase` for types and methods, and project namespaces such as `Lukas.Domain.Tests`. Vue and TypeScript use ESLint flat config plus Prettier; keep Vue components in `PascalCase.vue`, tests as `*.spec.ts`, and generated API files under `frontend/app/src/api/generated` untouched except through generation commands.

## Testing Guidelines

Backend tests use xUnit with Shouldly and live beside their target layer in `backend/tests`. Prefer focused test names that describe behavior, for example `SuccessResultExposesValue`. Frontend unit tests use Vitest and Vue Test Utils next to components, for example `BaseButton.spec.ts`. E2E coverage uses Playwright specs in `tests/e2e/specs`. Run `pnpm test` before opening a PR; run `pnpm test:e2e` when changing routing, app startup, or user-facing flows.

## Commit & Pull Request Guidelines

The current Git history uses short imperative or descriptive messages, for example `Update README.md` and `finished technical details`. Keep commits concise and scoped to one change. Pull requests should include a brief summary, test results (`pnpm check`, `pnpm test`, and e2e when relevant), linked issues or docs, and screenshots for visible UI changes.

## Security & Configuration Tips

Do not commit secrets, bank credentials, or local database files. Keep environment-specific API settings in `backend/src/Lukas.Api/appsettings.Development.json` or local overrides, and document any new required configuration in `README.md`.
