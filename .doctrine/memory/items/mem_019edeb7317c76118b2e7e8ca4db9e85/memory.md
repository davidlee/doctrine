# SL-109 string-building under clippy format_push_string and unwrap deny

Building strings with push_str vs format_push_string clippy lint. Repo bans unwrap/expect so write!/writeln! on String can't consume fmt::Result. Use multiple push_str calls instead of push_str(&format!(...)). SL-109 PHASE-02.
