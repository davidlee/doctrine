# claude plugin marketplace add overwrites source in place; refresh needs no remove+add

On CC 2.1.198, `claude plugin marketplace add <src>` overwrites an existing marketplace name's source when src differs (exit 0, "Successfully added"); idempotent no-op when src identical ("already on disk"). Stale-source refresh is a single add — no remove+add.
