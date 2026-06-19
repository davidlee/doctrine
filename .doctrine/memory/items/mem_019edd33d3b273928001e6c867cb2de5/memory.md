# SL-102 close: --integrate without --trunk is a dry run

dispatch sync --integrate without --trunk is a dry run — code does NOT land on main. At close, --trunk is mandatory; verify with git diff after integrate.
