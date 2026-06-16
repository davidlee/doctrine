# No standalone plan validation — malformed plan.toml surfaces late

There is no standalone plan.toml validation. A malformed plan.toml only surfaces when slice phases parses it at execution time. The author gets no feedback at plan-authoring time.
