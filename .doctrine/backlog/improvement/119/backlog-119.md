# IMP-119: Stable kebab-case Display formatter for CoverageStatus

## Context

`CoverageStatus` renders via `Debug`-as-display at `status_label` and
`withdrawal_line`. Replace with a single shared kebab-case `Display` impl so the
status label is stable and defined in one place.
