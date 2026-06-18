# SL-094: stale-closure pattern — parseTransform() decouples event handlers from closure-captured viewport

.dataset.zoomWired guard prevents re-wiring but creates stale-closure problem; parseTransform() reads viewport from CSS transform each event instead
