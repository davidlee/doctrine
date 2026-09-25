# ISS-482: A finding's blocking null reads as absent

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

A design-run finding declaration's `blocking` key is `Option<bool>` on the wire
(`src/design_run/submission.rs`, the `Declaration.blocking` field): `null`
deserialises to `None`, which reads as *absent*, which defaults to non-blocking.
So `blocking: null` on a finding succeeds and means "not blocking" — input the
engine did not honour as written. `SL-259`'s class (*success for input it
ignored*), the sibling of `ISS-481`'s `needs: null`.

Surfaced by `RV-386` `F-9` while designing `SL-264`, which gives `blocking` a
second home at inquiry nodes and refuses `null` there. The finding home is
outside `SL-264`'s surface, so it is left as found. Fix direction: parse the key
as `Sparse<bool>` and refuse `Null` at the finding home too.
