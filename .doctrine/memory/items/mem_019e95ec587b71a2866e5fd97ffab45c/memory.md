# Repo clippy denies: print_stdout, let_underscore_must_use, unused/dead_code

Repo denies print_stdout, let_underscore_must_use, unused/dead_code. Use writeln!(io::stdout()), push_str (not let _ = write!), #[expect(dead_code, reason)] for fields a later phase reads. Gate is plain 'cargo clippy', not --all-targets.
