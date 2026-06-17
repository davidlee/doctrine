# dispatch coordination branch may be GCd before audit — no lifecycle guard prevents premature removal. Observed on SL-085 where dispatch/085 was absent at audit time. The branch should survive until after reconciliation.


