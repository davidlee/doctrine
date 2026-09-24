`design_run`: `pass_stale = review.pass.is_some() && !review_standing().integrated_current`, and `integrated_current` is `ReviewPass::is_current(current_section_fingerprints)`.

So `pass_stale` is **true by construction** after you integrate a review's findings, because editing a section moves its fingerprint. It is an envelope lamp (`render/envelope.rs`), not a gate cause: the `reviewing → locked` edge's causes are `PassSuperseded` (the disposition names a different RV than the run's current pass), `ReviewUnavailable`, and `BlockersUndisposed` — none of which is pass currency.

Practical upshot: raise findings on the run's current pass RV, fix the design, dispose every finding, then dispose the *pass* with `conducted { review: <same RV> }`. A STALE lamp is expected and is not a reason to open a second RV (which would not be accepted anyway).
