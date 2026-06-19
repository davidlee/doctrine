# Dispatch orchestrator manual fixes after cherry-pick must be re-staged before commit

When the orchestrator cherry-picks a worker commit then manually edits files, the edits are in the working tree only — git commit picks up staged changes. Must git add after manual edits.
