use language (version is v0.1)
# This connection protocol has already grown beyond a binary fact: a client can
# be idle, authenticating, ready, or closed. A Boolean hides those alternatives
# and cannot make an invalid operation impossible.
connection-protocol : Boolean is false
