# pi v0.79.6 RPC: set_auto_retry command uses direct command type

The RPC `set_auto_retry` command format is:
```json
{"type":"set_auto_retry","enabled":false}
```

NOT:
```json
{"type":"request","method":"set_auto_retry","params":{"enabled":false}}
```

pi v0.79.6 responds to the wrong format with `{"type":"response","command":"request","success":false,"error":"Unknown command: request"}`.

This is per the RPC docs at `docs/rpc.md`: all commands are direct `type` values
(`prompt`, `set_auto_retry`, `get_state`, etc.), not wrapped in a `request`
envelope with `method`+`params`.

The design.md SL-108 spawn template uses the wrong format. Needs correction.
