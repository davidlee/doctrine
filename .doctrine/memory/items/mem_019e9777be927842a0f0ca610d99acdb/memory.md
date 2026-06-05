# Suppress lints with expect+reason, never bare allow

Bare #[allow] is denied; suppress lints with #[expect(.., reason)], gate test-used/prod-dead items via cfg_attr(not(test), expect(dead_code, reason))
