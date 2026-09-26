`DesignId` derives `Ord` over its string (`src/design_run/ids.rs:157`), and `design materialise` writes sections in that order: `sec-1, sec-10, sec-11, sec-12, sec-2, …`. The apply contract has no order key and no section removal.

Workaround used in SL-268: declare every body under the id whose lexical position matches its intended reading position (e.g. "Current state" as `sec-10`), and remap in-body `sec-N` cross-references in the same submission. Cheaper: keep a design to ≤9 sections. A friction observation was recorded (2026-09-26); fix is numeric ordering on the suffix.
