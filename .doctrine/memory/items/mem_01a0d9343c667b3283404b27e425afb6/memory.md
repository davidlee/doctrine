# textwrap splits at hyphens by default — use `NoHyphenation` for ids

`textwrap::Options::new(w)` defaults to `WordSplitter::HyphenSplitter`, which
breaks a word **at its existing hyphens** even with `break_words(false)`.
`break_words` only governs splitting a word wider than the line; the splitter
is a separate axis. For text carrying hyphenated identifiers (`inq-default-run`,
`SL-266`, paths), a wrapped line could end mid-id.

**How to apply:** when wrapping must never split a word, set both:

```rust
Options::new(width.max(1))
    .break_words(false)
    .word_splitter(WordSplitter::NoHyphenation)
```

Measure columns with `textwrap::core::display_width` (unicode-width is on in
this repo's textwrap features) — no direct `unicode-width` dependency needed.
Precedent: `src/design_run/render/tree.rs` `wrap()` (SL-266 PHASE-02).
