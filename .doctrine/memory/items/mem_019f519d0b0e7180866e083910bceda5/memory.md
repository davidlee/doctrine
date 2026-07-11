# Jail breaks lint-js: gate recipes fail on eslint /usr/bin/env shebang

just gate / doctrine check gate fail in the bubblewrap jail at lint-js (web/map eslint shebang needs /usr/bin/env, absent in jail) — env breakage, not a work defect; rust gate (just lint, fmt-check, cargo test) is the in-jail gate.
