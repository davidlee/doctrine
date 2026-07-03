# Idempotency comparator must track the emitted value: when a planner's output switches input-derived → constant, switch the no-op check too

A refresh planner's no-op branch must compare existing state against the value it now WRITES, not the old input; a constant emit with an input-derived comparator thrashes forever.
