# resolve_inputs pure-leaf via injected ResolveEnv (jail.rs)

Impure jail resolve_inputs stays a pure leaf via an injected ResolveEnv trait; all git/getconf/fs behind RealEnv
