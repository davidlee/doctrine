# Don't compromise doctrine-as-product for project-local operational concerns

Keep shipped doctrine strict/clean; fix transient project-local state out-of-band, never bake leniency into the product.

Durable design principle (user, SL-060 C6). When a transient project-local data/operational condition — e.g. existing entities lack a newly-seeded TOML table — tempts permanent leniency or complexity in the shipped library, refuse it. Keep the durable code strict and clean; fix the transient local state out-of-band.

Precedent: SL-048 "the cut" migrated tier-1 storage via a throwaway one-time corpus rewrite (round-trip `show` + `validate` verified), **zero** permanent migration surface — doctrine is the product, this repo is just its first client.

Apply: strict scaffold-guaranteed invariants + F-1 refuse stay uniform; out-of-band backfill for pre-existing entities, committed as a data-only diff. Don't ship a `migrate` verb or create-on-absent leniency to serve a one-time local gap.
