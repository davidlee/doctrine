# Dispatch integrate: clean exclusive trunk checkout, or the phantom index gets committed

Stage-2 integrate must run against a clean, exclusively-held trunk checkout; against a dirty shared tree it leaves a phantom index that the next commit reverts. On ISS-030 STOP: resync the checkout to HEAD before ANY commit.
