"""GDB pretty-printers for Topal's private native runtime values."""

import gdb


def _decimal_from_limbs(raw, length):
    """Convert little-endian u32 limbs without Python's integer-string cap."""
    decimal_chunks = [0]
    for index in range(length - 1, -1, -1):
        limb = int.from_bytes(raw[index * 4 : index * 4 + 4], "little")
        carry = limb
        for chunk_index, chunk in enumerate(decimal_chunks):
            current = chunk * 4_294_967_296 + carry
            decimal_chunks[chunk_index] = current % 1_000_000_000
            carry = current // 1_000_000_000
        while carry:
            decimal_chunks.append(carry % 1_000_000_000)
            carry //= 1_000_000_000
    return str(decimal_chunks[-1]) + "".join(
        f"{chunk:09d}" for chunk in reversed(decimal_chunks[:-1])
    )


class _TopalIntPrinter:
    """Render an immutable topal-native/3 Int object as a decimal integer."""

    def __init__(self, value):
        self._value = value

    def to_string(self):
        address = int(self._value)
        if address == 0:
            return "<invalid null Int>"
        inferior = gdb.selected_inferior()
        try:
            header = bytes(inferior.read_memory(address, 16))
        except gdb.MemoryError:
            return "<unreadable Int>"
        negative = int.from_bytes(header[0:8], "little")
        length = int.from_bytes(header[8:16], "little")
        if negative not in (0, 1):
            return f"<invalid Int sign {negative}>"
        if not length and negative:
            return "<noncanonical negative zero Int>"
        # Refuse an unreasonable interactive rendering request before GDB reads it.
        if length > 1_000_000:
            return f"<Int too large to render safely: {length} limbs>"
        magnitude = "0"
        if length:
            try:
                raw = bytes(inferior.read_memory(address + 16, length * 4))
            except gdb.MemoryError:
                return "<unreadable Int limbs>"
            if raw[-4:] == b"\0\0\0\0":
                return "<noncanonical Int leading zero limb>"
            magnitude = _decimal_from_limbs(raw, length)
        if negative and magnitude != "0":
            return "-" + magnitude
        return magnitude


class _TopalRationalPrinter:
    """Render a canonical topal-native Rational object."""

    def __init__(self, value):
        self._value = value

    def to_string(self):
        address = int(self._value)
        if address == 0:
            return "<invalid null Rational>"
        inferior = gdb.selected_inferior()
        try:
            header = bytes(inferior.read_memory(address, 16))
        except gdb.MemoryError:
            return "<unreadable Rational>"
        numerator_address = int.from_bytes(header[0:8], "little")
        denominator_address = int.from_bytes(header[8:16], "little")
        if not numerator_address or not denominator_address:
            return "<invalid null Rational component>"
        numerator = _TopalIntPrinter(numerator_address).to_string()
        denominator = _TopalIntPrinter(denominator_address).to_string()
        if numerator.startswith("<"):
            return f"<invalid Rational numerator: {numerator}>"
        if denominator.startswith("<"):
            return f"<invalid Rational denominator: {denominator}>"
        try:
            denominator_header = bytes(
                inferior.read_memory(denominator_address, 16)
            )
        except gdb.MemoryError:
            return "<unreadable Rational denominator>"
        denominator_negative = int.from_bytes(denominator_header[0:8], "little")
        denominator_length = int.from_bytes(denominator_header[8:16], "little")
        if denominator_negative or not denominator_length:
            return "<noncanonical Rational denominator>"
        return f"Rational ( {numerator}, {denominator} )"


def _lookup_topal_value(value):
    if str(value.type) == "Int":
        return _TopalIntPrinter(value)
    if str(value.type) == "Rational":
        return _TopalRationalPrinter(value)
    return None


gdb.pretty_printers.append(_lookup_topal_value)
