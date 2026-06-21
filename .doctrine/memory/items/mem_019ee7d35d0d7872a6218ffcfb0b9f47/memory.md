# Gate pre-filter: crate-root type edges are not module edges

When relocating enums from main.rs to kind modules, crate-root type refs (Command, CommonListArgs) become edges the gate can't classify. Pre-filter by source-file existence check.
