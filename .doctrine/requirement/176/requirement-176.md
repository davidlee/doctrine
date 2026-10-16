# REQ-176: Install Claude as a derived canonical skills tree with relative symlinks owned when their target lies inside a framework root

## Statement

A symlink in a registered link dir whose target lies inside a canon root is
doctrine's and is relinked to the current layout or removed; a stale-layout
link is healed; a non-symlink or a symlink elsewhere is kept untouched.

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->
