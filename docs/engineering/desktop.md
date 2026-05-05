# Desktop engineering

Status: Accepted

The desktop app lives in `apps/desktop` and is a Tauri shell around the same Vue
app used for browser development.

Tauri owns the desktop lifecycle: start the .NET sidecar, detect startup
failure, stop the sidecar on app exit and expose a local error state when the
API is unavailable.

## Sidecar

- Package the sidecar as a self-contained .NET executable for end-user desktop
  builds.
- Allow framework-dependent execution for local development.
- Bind to `127.0.0.1` on a dynamically selected port.
- Do not bind the sidecar to `0.0.0.0`.
- Pass runtime values through command-line arguments or environment variables.
- Store database files and logs under per-user app-data paths.

Runtime argument shape:

```text
--urls http://127.0.0.1:{port}
--database-path <app-data>/lukas.db
--log-directory <app-data>/logs
--profile desktop
```

Health checks:

```text
GET /health/live
GET /health/ready
```

Browser development mode remains the primary local workflow inside WSL. Tauri
desktop testing may need a Windows host, native Linux desktop session or another
environment with GUI support.

