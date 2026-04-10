# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-04-10

### Added
- `porygon-a2a-types`: A2A v1.0 protocol types auto-generated from official protobuf via prost + pbjson
- `porygon-a2a-server`: A2A server with `A2AHandler` trait and axum JSON-RPC router
  - All 11 A2A v1.0 methods (message, tasks, push notifications, extended card)
  - SSE streaming with JSON-RPC envelope and keep-alive
  - Error codes aligned with A2A v1.0 spec (-32001 through -32009)
  - A2A-Version header validation
- `porygon-a2a-client`: A2A HTTP client with reqwest
  - Blocking and SSE streaming support
  - Bearer token authentication
- `porygon-ag-ui-motosan`: AG-UI adapter for motosan-agent-loop
