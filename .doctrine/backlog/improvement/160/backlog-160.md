# IMP-160: strip HTML comments from boot-footer.md before injecting into boot snapshot

The `SourceKind::Footer` producer reads `.doctrine/boot-footer.md` verbatim
and injects its body into the boot snapshot under `## Onboarding`. But the
install template (`install/boot-footer.md`) carries HTML comments (`<!-- ... -->`)
that get passed through to the snapshot — noise for the agent.

Strip HTML comments from the file body before injecting. A simple regex or
line filter in the `produce()` arm for `SourceKind::Footer`.

## Implementation sketch

In `src/boot.rs`, the `SourceKind::Footer` arm in `produce()` currently:

```rust
SourceKind::Footer => section_or_marker(
    heading,
    fs::read_to_string(root.join(FOOTER_REL)).map_err(anyhow::Error::from),
),
```

Add a helper `strip_html_comments(s: &str) -> String` (pure, testable) that
removes `<!-- ... -->` blocks, then pipe the body through it before passing
to `section_or_marker`.

The helper should handle multi-line comments and not be over-eager (no removal
of `<!--` inside code fences, though that edge case is unlikely in this file).

## Related

- IMP-159: implemented the boot-footer.md mechanism
