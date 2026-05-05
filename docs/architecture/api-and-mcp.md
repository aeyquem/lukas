# API and MCP

Status: Accepted

The local desktop API is an internal command/query boundary, not a supported
public integration surface.

Public API and MCP access are deferred from the initial desktop PoC. Future
external access should start read-only, use scoped permissions and avoid
exposing bank credentials, provider tokens or raw integration secrets.

Selected write workflows may be allowed later through explicit commands, such
as uploading a bank statement that the main UI cannot map automatically. Any
external write must be auditable and reversible at the product level.

