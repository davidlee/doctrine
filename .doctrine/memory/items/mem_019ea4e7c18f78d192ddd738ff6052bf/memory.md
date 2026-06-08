# In the jail, cargo build writes to ~/.cargo/doctrine-target-jail/debug — ./target/debug/doctrine is stale

Embed-dependent runs must use the jail target binary, not ./target/debug/doctrine (a separate stale file)
