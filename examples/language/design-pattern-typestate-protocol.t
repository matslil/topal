use language (version is v0.1)
# A meter client has a small protocol. The Union makes alternatives and checks
# explicit, but this is only a runtime model: current Topal cannot consume a
# Ready capability when `read` is called or prevent callers from retaining it.
MeterProtocol is Union
  Idle : Int
  Authenticating : Int
  Ready : Int
  Closed : Int

authentication-result is fn (retry-count : Int, accepted : Boolean) -> MeterProtocol
  accepted
    true then Ready retry-count
    false then Closed retry-count

authenticate is fn (state : MeterProtocol, accepted : Boolean) -> MeterProtocol
  state
    Idle retry-count then Authenticating retry-count
    Authenticating retry-count then authentication-result (retry-count, accepted)
    Ready reading-count then Ready reading-count
    Closed retry-count then Closed retry-count

state : MeterProtocol is Idle 0
pending is authenticate (state, false)
authenticate (pending, true)
