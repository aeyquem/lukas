# Container Deployment Specification

## Purpose

Defines the local self-hosted development path and the independent container
interfaces used to deploy the frontend and backend without Docker Compose.

## Requirements

### Requirement: Local development uses the accepted private service topology
Local development SHALL provide PostgreSQL, API, Vite, and development-proxy services. Only the proxy SHALL publish a host port, bound to `127.0.0.1:4173`; PostgreSQL, API, and Vite SHALL remain on the private Compose network. PostgreSQL data SHALL persist across service restarts.

#### Scenario: Developer starts the local stack
- **WHEN** the developer starts the Compose development topology
- **THEN** the browser uses `http://localhost:4173` and only the loopback-bound proxy is reachable from the host

#### Scenario: A service restarts
- **WHEN** PostgreSQL or the API container restarts
- **THEN** PostgreSQL data remains available and the API can become ready again after reconnecting

### Requirement: Development proxy preserves the canonical browser boundary
The development proxy SHALL accept only Host `localhost:4173` and forward HTTP and HMR WebSocket traffic to Vite. Vite SHALL forward `/api` and `/health` paths unchanged to the private API while preserving the original Origin. SPA fallback SHALL apply only to frontend routes, never `/api` or `/health`.

#### Scenario: Valid browser request uses the API
- **WHEN** a browser at the canonical origin requests an `/api` route
- **THEN** the development proxy forwards the request to Vite, which forwards the unchanged path and Origin to the private API service

#### Scenario: Invalid Host is sent
- **WHEN** a request uses a Host outside the canonical value
- **THEN** the proxy rejects the request

#### Scenario: Frontend route and API path are requested
- **WHEN** the browser requests a client-side route, an `/api` path, or a `/health` path
- **THEN** client-side routes receive the SPA entry point and API/health paths reach the backend without SPA rewriting

#### Scenario: Vite HMR connects
- **WHEN** the browser opens the development HMR WebSocket through the proxy
- **THEN** the WebSocket connection reaches Vite and remains usable for hot reload

### Requirement: Production frontend and backend images are independent
The project SHALL build a backend image and a static frontend image that can be deployed independently. The production frontend SHALL serve SPA routes, proxy `/api` to a configured HTTP(S) `API_UPSTREAM`, and load a public non-secret `/runtime-config.js` generated from a validated `API_BASE_URL` before application bootstrap. Configuration values SHALL be safely encoded and SHALL NOT expose secrets. Production use SHALL NOT require Docker Compose or a Node application server.

#### Scenario: Frontend image is deployed independently
- **WHEN** the static frontend image starts with a configured `API_UPSTREAM` and `API_BASE_URL`
- **THEN** it serves the same built assets, provides SPA route fallback, and exposes runtime API configuration before the app starts

#### Scenario: Backend image is deployed independently
- **WHEN** the backend image starts with valid backend runtime configuration and PostgreSQL access
- **THEN** it runs without requiring the frontend image or Docker Compose

#### Scenario: Runtime API configuration is loaded
- **WHEN** a browser loads the frontend entry point
- **THEN** the public runtime configuration script is loaded before application bootstrap and contains no secret values

#### Scenario: Runtime frontend configuration is invalid
- **WHEN** `API_UPSTREAM` or `API_BASE_URL` contains an invalid URL or unsafe characters
- **THEN** the frontend container fails before serving a generated configuration or Nginx config

#### Scenario: Images run independently
- **WHEN** either production image is started without Docker Compose and with its required dependencies configured
- **THEN** the frontend serves its static app or the backend connects to PostgreSQL without requiring the other application image

### Requirement: Unauthenticated deployment remains same-machine only
Independent images SHALL NOT imply an approved unauthenticated LAN or public deployment. Until a separately reviewed access-control design exists, the supported topology SHALL publish the frontend only on loopback and keep the backend private behind it.

#### Scenario: Local unauthenticated deployment is started
- **WHEN** the supported local topology is started
- **THEN** only the frontend proxy is published on loopback and no backend port is published to the host

#### Scenario: Wider unauthenticated access is configured
- **WHEN** an operator attempts to expose the unauthenticated frontend or backend beyond the local machine
- **THEN** the setup documentation directs them to stop and obtain a separately reviewed access-control design
