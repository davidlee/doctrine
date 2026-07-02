# Neutralize lifecycle status on the dispatch code branch before building a close_target

Lifecycle status committed on a dispatch/code branch conflicts the close_target 3-way merge; neutralize it to the merge ancestor before prepare-review.
