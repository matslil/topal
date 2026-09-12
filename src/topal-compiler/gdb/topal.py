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
    """Render an immutable topal-native/5 Int object as a decimal integer."""

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


class _TopalRangePrinter:
    """Render a bounded finite exact topal-native Range object."""

    def __init__(self, value, endpoint):
        self._value = value
        self._endpoint = endpoint

    def to_string(self):
        address = int(self._value)
        if address == 0:
            return "<invalid null Range>"
        inferior = gdb.selected_inferior()
        try:
            header = bytes(inferior.read_memory(address, 32))
        except gdb.MemoryError:
            return "<unreadable Range>"
        lower_address = int.from_bytes(header[0:8], "little")
        upper_address = int.from_bytes(header[8:16], "little")
        lower_inclusive = int.from_bytes(header[16:24], "little")
        upper_inclusive = int.from_bytes(header[24:32], "little")
        if not lower_address or not upper_address:
            return "<invalid null Range endpoint>"
        if lower_inclusive not in (0, 1) or upper_inclusive not in (0, 1):
            return "<invalid Range inclusivity>"
        printer = (
            _TopalIntPrinter
            if self._endpoint == "Int"
            else _TopalRationalPrinter
        )
        lower = printer(lower_address).to_string()
        upper = printer(upper_address).to_string()
        if lower.startswith("<"):
            return f"<invalid Range lower endpoint: {lower}>"
        if upper.startswith("<"):
            return f"<invalid Range upper endpoint: {upper}>"
        symbol = {
            (1, 0): "..",
            (0, 0): "<..",
            (1, 1): "..=",
            (0, 1): "<..=",
        }[(lower_inclusive, upper_inclusive)]
        return f"{lower} {symbol} {upper}"


class _TopalErrorPrinter:
    """Render a structured topal-native arithmetic Error object."""

    def __init__(self, address):
        self._address = address

    def to_string(self):
        address = int(self._address)
        if address == 0:
            return "<invalid null Error>"
        inferior = gdb.selected_inferior()
        try:
            header = bytes(inferior.read_memory(address, 80))
        except gdb.MemoryError:
            return "<unreadable Error>"
        domain_address = int.from_bytes(header[0:8], "little")
        domain_length = int.from_bytes(header[8:16], "little")
        code = int.from_bytes(header[16:20], "little")
        if not domain_address or domain_length > 1_000_000:
            return "<invalid Error domain>"
        try:
            domain = bytes(
                inferior.read_memory(domain_address, domain_length)
            ).decode("utf-8")
        except (gdb.MemoryError, UnicodeDecodeError):
            return "<unreadable Error domain>"
        codes = {
            0: "out-of-range",
            1: "not-representable",
            2: "division-by-zero",
            3: "indeterminate",
        }
        if code not in codes:
            return f"<invalid arithmetic Error code {code}>"
        return f"Error ( domain is {domain}, code is {codes[code]} )"


class _TopalResultPrinter:
    """Render a topal-native Result through its statically known success type."""

    def __init__(self, value, success):
        self._value = value
        self._success = success

    def to_string(self):
        address = int(self._value)
        if address == 0:
            return "<invalid null Result>"
        inferior = gdb.selected_inferior()
        try:
            header = bytes(inferior.read_memory(address, 16))
        except gdb.MemoryError:
            return "<unreadable Result>"
        tag = int.from_bytes(header[0:8], "little")
        payload = int.from_bytes(header[8:16], "little")
        if tag == 1:
            return _TopalErrorPrinter(payload).to_string()
        if tag != 0:
            return f"<invalid Result tag {tag}>"
        if self._success in ("Int", "Nat"):
            return _TopalIntPrinter(payload).to_string()
        if self._success == "Rational":
            return _TopalRationalPrinter(payload).to_string()
        if self._success == "(Int, Int)":
            try:
                pair = bytes(inferior.read_memory(payload, 16))
            except gdb.MemoryError:
                return "<unreadable Result success pair>"
            quotient = int.from_bytes(pair[0:8], "little")
            remainder = int.from_bytes(pair[8:16], "little")
            return (
                f"({_TopalIntPrinter(quotient).to_string()}, "
                f"{_TopalIntPrinter(remainder).to_string()})"
            )
        return f"<unsupported Result success type {self._success}>"


def _lookup_topal_value(value):
    value_type = str(value.type)
    if value_type == "Int":
        return _TopalIntPrinter(value)
    if value_type == "Nat":
        return _TopalIntPrinter(value)
    if value_type == "Rational":
        return _TopalRationalPrinter(value)
    if value_type == "Range Int":
        return _TopalRangePrinter(value, "Int")
    if value_type == "Range Rational":
        return _TopalRangePrinter(value, "Rational")
    prefix = "Result ("
    suffix = ", lang arithmetic ArithmeticErrorCode)"
    if value_type.startswith(prefix) and value_type.endswith(suffix):
        return _TopalResultPrinter(value, value_type[len(prefix) : -len(suffix)])
    return None


gdb.pretty_printers.append(_lookup_topal_value)
