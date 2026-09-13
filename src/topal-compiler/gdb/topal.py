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
    """Render an immutable topal-native/6 Int object as a decimal integer."""

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


def _display_string(value):
    if '"' not in value:
        return f'"{value}"'
    tag = "text"
    while '"' + tag in value:
        tag += "_"
    return f'{tag}"{value}"{tag}'


class _TopalStringPrinter:
    """Render an immutable UTF-8 topal-native String descriptor."""

    def __init__(self, value, quoted=True):
        self._value = value
        self._quoted = quoted

    def to_string(self):
        address = int(self._value)
        if address == 0:
            return "<invalid null String>"
        inferior = gdb.selected_inferior()
        try:
            header = bytes(inferior.read_memory(address, 16))
        except gdb.MemoryError:
            return "<unreadable String>"
        data_address = int.from_bytes(header[0:8], "little")
        length = int.from_bytes(header[8:16], "little")
        if length > 1_000_000 or (length and not data_address):
            return "<invalid String storage>"
        value = ""
        if length:
            try:
                value = bytes(inferior.read_memory(data_address, length)).decode("utf-8")
            except (gdb.MemoryError, UnicodeDecodeError):
                return "<unreadable String data>"
        return _display_string(value) if self._quoted else value


class _TopalErrorCodePrinter:
    """Render the closed arithmetic ErrorCode vocabulary."""

    def __init__(self, value):
        self._value = value

    def to_string(self):
        codes = {
            0: "out-of-range",
            1: "not-representable",
            2: "division-by-zero",
            3: "indeterminate",
        }
        code = int(self._value)
        return codes.get(code, f"<invalid arithmetic Error code {code}>")


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
            header = bytes(inferior.read_memory(address, 56))
        except gdb.MemoryError:
            return "<unreadable Error>"
        domain_address = int.from_bytes(header[0:8], "little")
        code = int.from_bytes(header[8:12], "little")
        if not domain_address:
            return "<invalid Error domain>"
        domain = _TopalStringPrinter(domain_address, quoted=False).to_string()
        code = _TopalErrorCodePrinter(code).to_string()
        if domain.startswith("<"):
            return f"<invalid Error domain: {domain}>"
        if code.startswith("<"):
            return code
        return f"Error ( domain is {domain}, code is {code} )"


class _TopalSourceLocationPrinter:
    """Render the source-visible line and column of an Error."""

    def __init__(self, address):
        self._address = address

    def to_string(self):
        address = int(self._address)
        if address == 0:
            return "<invalid null SourceLocation>"
        inferior = gdb.selected_inferior()
        try:
            header = bytes(inferior.read_memory(address, 16))
        except gdb.MemoryError:
            return "<unreadable SourceLocation>"
        line = int.from_bytes(header[0:8], "little")
        column = int.from_bytes(header[8:16], "little")
        if not line or not column:
            return "<invalid SourceLocation field>"
        line = _TopalIntPrinter(line).to_string()
        column = _TopalIntPrinter(column).to_string()
        return f"(line is {line}, column is {column})"


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
        if self._success == "String":
            return _TopalStringPrinter(payload).to_string()
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


class _TopalOptionalPrinter:
    """Render a topal-native Optional through its statically known payload type."""

    def __init__(self, value, payload_type):
        self._value = value
        self._payload_type = payload_type

    def to_string(self):
        address = int(self._value)
        if address == 0:
            return "<invalid null Optional>"
        inferior = gdb.selected_inferior()
        try:
            header = bytes(inferior.read_memory(address, 16))
        except gdb.MemoryError:
            return "<unreadable Optional>"
        tag = int.from_bytes(header[0:8], "little")
        payload = int.from_bytes(header[8:16], "little")
        if tag == 0:
            if payload:
                return "<invalid None payload>"
            return "None"
        if tag != 1:
            return f"<invalid Optional tag {tag}>"
        if not payload:
            return "<invalid null Some payload>"
        if self._payload_type == "Int":
            rendered = _TopalIntPrinter(payload).to_string()
        elif self._payload_type == "Rational":
            rendered = _TopalRationalPrinter(payload).to_string()
        elif self._payload_type in ("Character", "String"):
            rendered = _TopalStringPrinter(payload).to_string()
        elif self._payload_type == "Error":
            rendered = _TopalErrorPrinter(payload).to_string()
        elif self._payload_type == "SourceLocation":
            rendered = _TopalSourceLocationPrinter(payload).to_string()
        else:
            return f"<unsupported Optional payload type {self._payload_type}>"
        return f"Some {rendered}"


def _render_topal_value(value):
    printer = _lookup_topal_value(value)
    if printer is not None:
        return printer.to_string()
    value_type = value.type.strip_typedefs()
    try:
        fields = value_type.fields()
    except gdb.error:
        fields = ()
    if fields and all(field.name and field.name.startswith("_") for field in fields):
        rendered = [_render_topal_value(value[field.name]) for field in fields]
        suffix = "," if len(rendered) == 1 else ""
        return "(" + ", ".join(rendered) + suffix + ")"
    return str(value)


class _TopalSumPrinter:
    """Render a private aggregate while observing only its active sum payload."""

    def __init__(self, value):
        self._value = value

    def to_string(self):
        tag = self._value["tag"]
        index = int(tag)
        alternative = str(tag)
        valid_tags = {
            field.enumval for field in tag.type.strip_typedefs().fields()
        }
        if index not in valid_tags:
            return f"<invalid sum tag {index}>"
        prefix = alternative
        try:
            payload = self._value[f"payload_{index}"]
        except gdb.error:
            return prefix
        return f"{prefix} {_render_topal_value(payload)}"


def _lookup_topal_value(value):
    value_type = str(value.type)
    storage_type = str(value.type.strip_typedefs())
    if value_type == "Int":
        return _TopalIntPrinter(value)
    if value_type == "Nat":
        return _TopalIntPrinter(value)
    if storage_type == "struct TopalIntHeader *":
        return _TopalIntPrinter(value)
    if value_type == "Rational":
        return _TopalRationalPrinter(value)
    if value_type == "Character":
        return _TopalStringPrinter(value)
    if value_type == "String":
        return _TopalStringPrinter(value)
    if value_type == "Error":
        return _TopalErrorPrinter(value)
    if value_type == "lang arithmetic ArithmeticErrorCode":
        return _TopalErrorCodePrinter(value)
    if value_type == "ErrorDomain":
        return _TopalStringPrinter(value, quoted=False)
    if value_type == "SourceLocation":
        return _TopalSourceLocationPrinter(value)
    if value_type == "Range Int":
        return _TopalRangePrinter(value, "Int")
    if value_type == "Range Rational":
        return _TopalRangePrinter(value, "Rational")
    optional_prefix = "Optional "
    if value_type.startswith(optional_prefix):
        return _TopalOptionalPrinter(value, value_type[len(optional_prefix) :])
    prefix = "Result ("
    suffix = ", lang arithmetic ArithmeticErrorCode)"
    if value_type.startswith(prefix) and value_type.endswith(suffix):
        return _TopalResultPrinter(value, value_type[len(prefix) : -len(suffix)])
    if storage_type.startswith("struct TopalUnion."):
        return _TopalSumPrinter(value)
    if storage_type.startswith("struct TopalVariant."):
        return _TopalSumPrinter(value)
    return None


gdb.pretty_printers.append(_lookup_topal_value)
