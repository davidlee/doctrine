When changing `read_doc(g, gov_root, id)` and `set_status(g, gov_root, id, ...)` in `src/governance.rs` to use `entity::id_path` instead of manual format! path construction, the parameter changed from `gov_root` (kind tree root, e.g. `.doctrine/adr`) to `root` (project root).

`entity::id_path(root, &g.kind, id, Ext::Toml)` produces `root.join(g.kind.dir).join(format!("{:03}", id)).join(format!("{}-{:03}.toml", g.kind.stem, id))`.

ALL callers must pass the project root, not the kind root. Common patterns:
- Production: `&root` (from `crate::root::find(path, ...)` or function param)
- Tests: `root` (from `dir.path()`) — NOT `&adr_root(root)` or `&rfc_root(root)`

The doubled-path error pattern is: `.../.doctrine/adr/.doctrine/adr/001/adr-001.toml` — indicates a caller still passes `adr_root(root)`.

After changing `read_doc`/`set_status`, search for: `grep -rn 'read_doc(\|set_status(' src/` and verify each caller passes project root, not kind root.
