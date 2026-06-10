# Extending the shared list column model (listing.rs): pre-materialise the row, non-capturing fn extractors

To add a kind to the column model: build a typed table row, resolve runtime context (e.g. subtype-prefixed id) INTO it, keep extractors a const of non-capturing fn(&R)->String; columns.take() before build, select_columns once, render_columns per block; JSON stays per-kind typed.
