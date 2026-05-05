# Backend engineering

Status: Accepted

Backend code lives under `backend/src/Lukas.*` and tests under
`backend/tests/Lukas.*.Tests`.

## Projects

- `Lukas.Api`: ASP.NET API/local sidecar host.
- `Lukas.Domain`: core domain model and domain behavior.
- `Lukas.Application`: commands, queries and use-case orchestration.
- `Lukas.Infrastructure`: SQLite, files, logging, clocks and adapters.
- `Lukas.Importers`: CSV, Excel and external financial importers.
- `Lukas.Contracts`: API request/response contracts.
- `Lukas.SharedKernel`: low-level primitives such as `Result`, `Maybe`, `Unit`
  and `Error`.

`Lukas.Contracts` must not contain domain behavior, persistence attributes or
UI-specific models. `Lukas.SharedKernel` must not contain domain behavior, API
DTOs, persistence models or UI concepts.

## API

Use ASP.NET Core controllers for `Lukas.Api`. Controllers stay thin: they map
HTTP requests/responses, call application command/query handlers and translate
application errors into HTTP responses.

Use DataAnnotations for basic request-shape validation and standard
`ValidationProblemDetails` for invalid input DTOs. Business validation belongs
in `Lukas.Application` and `Lukas.Domain`. Expected application/domain failures
return `ProblemDetails`; unexpected failures are logged at the API boundary.

## Persistence

Use EF Core for initial SQLite persistence and migrations. Keep domain behavior
separate from persistence concerns. Use separate persistence models when EF
mapping would pollute domain shapes.

Core records use app-generated UUID-style stable IDs and include workspace
boundaries from the beginning. Keep migrations committed. Keep seed data
test/dev-only.

