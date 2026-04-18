# porygon-a2a-types

A2A v1.0 protocol types for Rust, auto-generated from the [official protobuf](https://github.com/a2aproject/A2A/blob/main/specification/a2a.proto) via `prost` + `pbjson`.

Part of the [porygon](https://github.com/motosan-dev/porygon) workspace.

## Usage

```toml
[dependencies]
porygon-a2a-types = "0.1"
```

The crate is imported as `a2a_types`:

```rust
use a2a_types::*;

let task = Task {
    id: "task-1".into(),
    status: Some(TaskStatus {
        state: TaskState::Completed as i32,
        ..Default::default()
    }),
    ..Default::default()
};

let json = serde_json::to_string(&task).unwrap();
```

All types support `serde::Serialize` and `serde::Deserialize` with JSON field names following the ProtoJSON specification.

## License

MIT
