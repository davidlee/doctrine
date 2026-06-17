# Routing row must follow install: install skill before adding to routing-process.md

When adding a skill to the routing table, install the skill (doctrine claude install) BEFORE adding the routing row — otherwise the boot snapshot points at a deferred skill, violating ADR-009 F14 shipped-not-reachable.
