# Changelog

All notable changes to this project will be documented in this file.

## [0.1.1] - 2026-04-18

### Changed
- Bumped `motosan-agent-loop` 0.9.4 → 0.13.1 and `motosan-agent-tool` 0.3 → 0.3.2
- `porygon-ag-ui-motosan`: adapted `translate()` to the new two-layer `AgentEvent` shape (`Core(CoreEvent)` / `Extension(ExtensionEvent)`) introduced upstream; `CoreEvent::IterationStarted` is now a struct variant

### Fixed
- READMEs: use published `porygon-*` crate names in `[dependencies]` examples (previously showed short names)

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
