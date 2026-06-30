# Predicate staged one phase ahead of its consumer: gate dead_code via cfg_attr(not(test), expect)

A pub(crate) fn used only by tests now (real consumer lands a later phase) needs cfg_attr(not(test), expect(dead_code)) — a bare #[expect] is unfulfilled under cargo test.
