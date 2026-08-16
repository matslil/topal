#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates legacy warning controls and severity-neutral controls using a
# structured diagnostic identity. Neither form changes execution decisions.
lang disable-warning example-warning
value is 41
lang push-disable-warning example-warning
lang pop-disable-warning example-warning

lang disable-diagnostic ( lang best-practice task state-machine )
unchanged is value
lang push-disable-diagnostic ( lang best-practice task state-machine )
lang pop-disable-diagnostic ( lang best-practice task state-machine )
value + 1
