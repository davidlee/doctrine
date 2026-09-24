`retrieve_rows(path, probe, fetch)` (`src/retrieve.rs`) expands `ScopeProbe` into `(paths: Vec<String>, commands: Vec<String>)`; `QueryContext.paths` is already a `Vec` and `match_scope` admits a memory when **any** query path matches (`locations().any(...)`), with one ranking across all of them.

Consequence: a multi-file input (e.g. a codex `apply_patch` touching N files) must ride a **path-set probe**, not N `retrieve_rows` calls. Looping costs one whole corpus load (`collect_all`) and one git capture (`git::capture`) per path, and awards the cap to iteration order rather than to rank.

Widening `ScopeProbe`'s path arm is an *arity* change, not new retrieval logic — no query, admission rule, ranking or tuning knob changes.
