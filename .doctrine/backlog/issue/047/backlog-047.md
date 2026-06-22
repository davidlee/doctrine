# ISS-047: `doctrine memory show` can't resolve shipped memories by key

`run_show()` only searches `.doctrine/memory/items/` via `resolve_show()`. The
shipped corpus (`shipped/`, populated by `memory sync`) stores memories as
uid-named directories without key symlinks. So `show` by key (e.g.
`mem.signpost.doctrine.overview`) misses shipped memories entirely.

`find`, `retrieve`, `show` by uid, and `collect_all` (used by wikilink
resolution) all union both namespaces correctly. `resolve_show` is the outlier.

**Fix**: make `resolve_show` fall through to shipped/ when items/ doesn't have
the name — same pattern as `read_body()` at memory.rs:2618.

```rust
// in resolve_show, after items/ lookup fails:
let shipped_dir = items_root.parent()  // memory/
    .map(|mem| mem.join(MEMORY_SHIPPED_DIR));
if let Some(shipped) = shipped_dir {
    let shipped_toml = shipped.join(&name).join("memory.toml");
    if shipped_toml.exists() {
        let memory = Memory::parse(&fs::read_to_string(&shipped_toml)?)?;
        let body = fs::read_to_string(shipped.join(&name).join("memory.md"))
            .unwrap_or_default();
        return Ok((memory, body, shipped.join(&name)));
    }
}
```

**Spotted while testing** IDE-020 (seed project-orientation memory during install).
The shipped overview (`mem.signpost.doctrine.overview`) couldn't be shown by key
after `memory sync`.
