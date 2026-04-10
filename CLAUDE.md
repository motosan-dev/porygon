# Porygon — Project Context

## What is this?

Rust workspace implementing agent communication protocols:
- **A2A** (Agent-to-Agent) v1.0 — JSON-RPC over HTTP with SSE streaming
- **AG-UI** adapter for motosan-agent-loop

## Workspace Structure

| Crate | crates.io name | Purpose |
|-------|---------------|---------|
| `crates/a2a-types/` | `porygon-a2a-types` | Proto-generated types (prost + pbjson) |
| `crates/a2a-server/` | `porygon-a2a-server` | A2AHandler trait + axum router |
| `crates/a2a-client/` | `porygon-a2a-client` | reqwest HTTP client |
| `crates/ag-ui-motosan/` | `porygon-ag-ui-motosan` | motosan AgentEvent adapter |

## Key Files

- `crates/a2a-types/proto/a2a.proto` — stripped version of official A2A proto (no google/api imports, no gRPC service)
- `crates/a2a-types/build.rs` — prost + pbjson codegen
- `crates/a2a-server/src/handler.rs` — `A2AHandler` trait with all 11 methods
- `crates/a2a-server/src/router.rs` — axum JSON-RPC dispatch + SSE streaming

## Build Requirements

- Rust stable
- `protoc` (protobuf compiler) — needed by `prost-build` for a2a-types

## Commands

```bash
cargo build          # build all crates
cargo test           # run all tests (31 total)
cargo clippy         # lint
cargo fmt            # format
```

## Publishing

Crates must be published in dependency order:
1. `porygon-a2a-types`
2. `porygon-a2a-client`
3. `porygon-a2a-server`
4. `porygon-ag-ui-motosan`

Tag a release `v0.1.0` to trigger `.github/workflows/publish.yml`.

## A2A Protocol Notes

- JSON-RPC 2.0 over HTTP, SSE for streaming
- Enum values use ProtoJSON SCREAMING_SNAKE_CASE (e.g., `TASK_STATE_COMPLETED`, `ROLE_USER`)
- Error codes: -32001 (TaskNotFound) through -32009 (VersionNotSupported)
- A2A-Version header validated on requests (supported: "1.0")

## Updating A2A Proto

```bash
curl -sL https://raw.githubusercontent.com/a2aproject/A2A/main/specification/a2a.proto \
  | sed '/^import "google\/api/d' \
  | sed '/option (google\.api\.http)/,/};/d' \
  | sed '/option (google\.api\.method_signature)/d' \
  | sed 's/ \[(google\.api\.field_behavior) = [A-Z_]*\]//g' \
  | sed '/^option csharp_namespace/d;/^option go_package/d;/^option java/d' \
  > crates/a2a-types/proto/a2a.proto
# Then manually remove the A2AService block and google.protobuf.Empty import
cargo build -p porygon-a2a-types
```
