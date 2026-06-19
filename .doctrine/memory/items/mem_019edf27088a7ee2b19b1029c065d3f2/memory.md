# Specs carry no authored date on disk

Specs are the only doctrine kind with no created/updated date on disk; a consumer
needing a spec date must derive it (e.g. toml mtime).

Slices, ADRs, and backlog items all carry authored `created`/`updated` in their
`*-NNN.toml`. **Specs do not** — their toml has identity/status/lineage but no date
field. Any code that maps a spec to a dated representation must supply the date from
outside the authored data.

Bit SL-026 (lazyspec projection): `head.created.unwrap_or_default()` silently emitted
`date: ""` for all 34 specs, breaking a downstream mandatory `%Y-%m-%d` parse. The
clean in-memory fixtures all set dates and hid it — only a live smoke over the real
corpus caught it. Resolution: inject the spec toml's filesystem **mtime** in the impure
shell (lossy-v1, checkout-unstable). The durable fix — authored dates on the spec
schema — is tracked as IMP-108.

Test lesson: a date-handling test over specs must seed a **real dateless scaffold**,
not a dated in-memory fixture, or it cannot catch this class.
