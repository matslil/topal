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
    """Render an immutable finite Int or executable-private infinity sentinel."""

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
        if negative in (2, 3):
            if length:
                return f"<invalid Infinity length {length}>"
            return "-Infinity" if negative == 3 else "+Infinity"
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


class _TopalModularPrinter:
    """Render a nominal modular value backed by a canonical Topal Int."""

    def __init__(self, value, name):
        self._value = value
        self._name = name

    def to_string(self):
        return f"{self._name} {_TopalIntPrinter(self._value).to_string()}"


class _TopalVersionPrinter:
    """Render a numeric Version from its four immutable Nat components."""

    def __init__(self, value):
        self._value = value

    def to_string(self):
        address = int(self._value)
        if address == 0:
            return "<invalid null Version>"
        inferior = gdb.selected_inferior()
        try:
            header = bytes(inferior.read_memory(address, 32))
        except gdb.MemoryError:
            return "<unreadable Version>"
        components = []
        for index, name in enumerate(("major", "minor", "patch", "build")):
            component = int.from_bytes(header[index * 8 : index * 8 + 8], "little")
            if not component:
                return f"<invalid null Version {name}>"
            rendered = _TopalIntPrinter(component).to_string()
            if rendered.startswith("<"):
                return f"<invalid Version {name}: {rendered}>"
            if rendered.startswith("-"):
                return f"<invalid negative Version {name}>"
            components.append(rendered)
        major, minor, patch, build = components
        if build != "0":
            return f"v{major}.{minor}.{patch}-{build}"
        if patch != "0":
            return f"v{major}.{minor}.{patch}"
        return f"v{major}.{minor}"


class _TopalSerializationStreamPrinter:
    """Render an immutable canonical Topal native serialization stream."""

    def __init__(self, value):
        self._value = value

    def to_string(self):
        address = int(self._value)
        if address == 0:
            return "<invalid null SerializationStream>"
        inferior = gdb.selected_inferior()
        try:
            header = bytes(inferior.read_memory(address, 16))
        except gdb.MemoryError:
            return "<unreadable SerializationStream>"
        data_address = int.from_bytes(header[0:8], "little")
        length = int.from_bytes(header[8:16], "little")
        if length > 16 * 1024 * 1024:
            return "<SerializationStream too large to render safely>"
        if length and not data_address:
            return "<invalid SerializationStream storage>"
        if length < 8:
            return "<truncated SerializationStream header>"
        try:
            magic = bytes(inferior.read_memory(data_address, 8))
        except gdb.MemoryError:
            return "<unreadable SerializationStream data>"
        if magic != b"TOPALSER":
            return "<invalid SerializationStream magic>"
        return f"SerializationStream ( {length} bytes )"


class _TopalTaskPrinter:
    """Render a private direct Task instance without exposing LLVM symbols."""

    def __init__(self, value, name):
        self._value = value
        self._name = name

    def to_string(self):
        address = int(self._value)
        if address == 0:
            return "<invalid null Task>"
        inferior = gdb.selected_inferior()
        try:
            header = bytes(inferior.read_memory(address, 24))
        except gdb.MemoryError:
            return "<unreadable Task>"
        identity = int.from_bytes(header[0:8], "little")
        terminated = int.from_bytes(header[8:16], "little")
        state = int.from_bytes(header[16:24], "little")
        if not identity:
            return "<invalid Task identity>"
        if terminated not in (0, 1):
            return "<invalid Task lifecycle state>"
        if not state:
            return "<invalid null Task state>"
        rendered = _TopalIntPrinter(state).to_string()
        if rendered.startswith("<"):
            return f"<invalid Task state: {rendered}>"
        state_name = "state"
        try:
            fields = self._value.type.strip_typedefs().target().fields()
            if len(fields) >= 3 and fields[2].name:
                state_name = fields[2].name
        except gdb.error:
            pass
        lifecycle = "terminated" if terminated else "active"
        return (
            f"{self._name} ( identity is {identity}, {lifecycle}, "
            f"{state_name} is {rendered} )"
        )


class _TopalLocationPrinter:
    """Render a checked process-owned external Location defensively."""

    def __init__(self, value, name):
        self._value = value
        self._name = name

    def to_string(self):
        address = int(self._value)
        if not address:
            return "<invalid null Location>"
        inferior = gdb.selected_inferior()
        try:
            header = bytes(inferior.read_memory(address, 32))
        except gdb.MemoryError:
            return "<unreadable Location>"
        range_start = int.from_bytes(header[0:8], "little")
        offset = int.from_bytes(header[8:16], "little")
        initialized = int.from_bytes(header[16:24], "little")
        stored = int.from_bytes(header[24:32], "little")
        if not range_start or not offset:
            return "<invalid Location address evidence>"
        if initialized not in (0, 1):
            return "<invalid Location initialization state>"
        range_start = _TopalIntPrinter(range_start).to_string()
        offset = _TopalIntPrinter(offset).to_string()
        if range_start.startswith("<") or offset.startswith("<"):
            return "<invalid Location address evidence>"
        if not initialized:
            if stored:
                return "<invalid uninitialized Location value>"
            state = "uninitialized"
        else:
            if not stored:
                return "<invalid null Location value>"
            value = _TopalIntPrinter(stored).to_string()
            if value.startswith("<"):
                return f"<invalid Location value: {value}>"
            state = f"value is {value}"
        return (
            f"{self._name} ( range-start is {range_start}, offset is {offset}, "
            f"{state} )"
        )


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


class _TopalGeneratorErrorCodePrinter:
    """Render the closed generator ErrorCode vocabulary."""

    def __init__(self, value):
        self._value = value

    def to_string(self):
        code = int(self._value)
        if code == 0:
            return "generator-closed"
        return f"<invalid generator Error code {code}>"


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
    """Render a structured topal-native Error object."""

    def __init__(self, address, code_vocabulary="arithmetic"):
        self._address = address
        self._code_vocabulary = code_vocabulary

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
        code_printer = (
            _TopalGeneratorErrorCodePrinter
            if self._code_vocabulary == "generator"
            else _TopalErrorCodePrinter
        )
        code = code_printer(code).to_string()
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


def _render_boxed_enum(payload, type_name, context):
    """Render a compiler-private boxed enum payload when DWARF retains its type."""

    try:
        enum_type = gdb.lookup_type(type_name).strip_typedefs()
    except gdb.error:
        try:
            enum_type = gdb.lookup_type(f"enum {type_name}").strip_typedefs()
        except gdb.error:
            return None
    if enum_type.code != gdb.TYPE_CODE_ENUM:
        return None
    if not payload:
        return f"<invalid null {context}>"
    try:
        encoded = bytes(gdb.selected_inferior().read_memory(payload, 4))
    except gdb.MemoryError:
        return f"<unreadable {context}>"
    value = int.from_bytes(encoded, "little")
    alternatives = {
        int(field.enumval): field.name for field in enum_type.fields()
    }
    if value not in alternatives:
        return f"<invalid {context} tag {value}>"
    return alternatives[value]


class _TopalResultPrinter:
    """Render a topal-native Result through its statically known success type."""

    def __init__(self, value, success, error_vocabulary="arithmetic"):
        self._value = value
        self._success = success
        self._error_vocabulary = error_vocabulary

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
            return _TopalErrorPrinter(payload, self._error_vocabulary).to_string()
        if tag != 0:
            return f"<invalid Result tag {tag}>"
        if self._success in ("Int", "Nat"):
            return _TopalIntPrinter(payload).to_string()
        if self._success == "Rational":
            return _TopalRationalPrinter(payload).to_string()
        if self._success == "String":
            return _TopalStringPrinter(payload).to_string()
        if self._success == "Unit":
            return "()"
        enum_rendered = _render_boxed_enum(
            payload, self._success, "Result success enum"
        )
        if enum_rendered is not None:
            return enum_rendered
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
        if self._success == "(Int, String)":
            try:
                pair = bytes(inferior.read_memory(payload, 16))
            except gdb.MemoryError:
                return "<unreadable Result success product>"
            integer = int.from_bytes(pair[0:8], "little")
            text = int.from_bytes(pair[8:16], "little")
            if not integer or not text:
                return "<invalid null Result success product field>"
            integer = _TopalIntPrinter(integer).to_string()
            text = _TopalStringPrinter(text).to_string()
            if integer.startswith("<") or text.startswith("<"):
                return f"<invalid Result success product: ({integer}, {text})>"
            return f"({integer}, {text})"
        try:
            success_type = gdb.lookup_type(self._success).strip_typedefs()
        except gdb.error:
            success_type = None
        if success_type is not None and str(success_type).startswith(
            "struct TopalModular."
        ):
            return _TopalModularPrinter(payload, self._success).to_string()
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
        list_prefix = "List "
        if self._payload_type.startswith(list_prefix):
            rendered = _TopalListPrinter(
                payload, self._payload_type[len(list_prefix) :]
            ).to_string()
            if rendered.startswith("<"):
                return f"<invalid Optional List payload: {rendered}>"
            return f"Some {rendered}"
        if self._payload_type.startswith("List("):
            rendered = _TopalListPrinter(
                payload, self._payload_type[len("List") :]
            ).to_string()
            if rendered.startswith("<"):
                return f"<invalid Optional List payload: {rendered}>"
            return f"Some {rendered}"
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
        elif self._payload_type == "(Int, List Int)":
            try:
                pair = bytes(inferior.read_memory(payload, 16))
            except gdb.MemoryError:
                return "<unreadable Optional List decomposition>"
            first = int.from_bytes(pair[0:8], "little")
            if not first:
                return "<invalid null List decomposition entry>"
            rest = int.from_bytes(pair[8:16], "little")
            first = _TopalIntPrinter(first).to_string()
            rest = _TopalListPrinter(rest, "Int").to_string()
            if first.startswith("<") or rest.startswith("<"):
                return f"<invalid Optional List decomposition: ({first}, {rest})>"
            rendered = f"({first}, {rest})"
        elif self._payload_type == "(Int, String)":
            try:
                pair = bytes(inferior.read_memory(payload, 16))
            except gdb.MemoryError:
                return "<unreadable Optional product>"
            integer = int.from_bytes(pair[0:8], "little")
            text = int.from_bytes(pair[8:16], "little")
            if not integer or not text:
                return "<invalid null Optional product field>"
            integer = _TopalIntPrinter(integer).to_string()
            text = _TopalStringPrinter(text).to_string()
            if integer.startswith("<") or text.startswith("<"):
                return f"<invalid Optional product: ({integer}, {text})>"
            rendered = f"({integer}, {text})"
        else:
            rendered = _render_boxed_enum(
                payload, self._payload_type, "Optional enum payload"
            )
            if rendered is None:
                return f"<unsupported Optional payload type {self._payload_type}>"
        return f"Some {rendered}"


class _TopalTraversalControlPrinter:
    """Render a topal-native Continue or Finish traversal result."""

    def __init__(self, value, payload_type):
        self._value = value
        self._payload_type = payload_type

    def to_string(self):
        address = int(self._value)
        if address == 0:
            return "<invalid null TraversalControl>"
        inferior = gdb.selected_inferior()
        try:
            storage = bytes(inferior.read_memory(address, 16))
        except gdb.MemoryError:
            return "<unreadable TraversalControl>"
        tag = int.from_bytes(storage[0:8], "little")
        payload = int.from_bytes(storage[8:16], "little")
        if tag not in (0, 1):
            return f"<invalid TraversalControl tag {tag}>"
        if not payload:
            return "<invalid null TraversalControl payload>"
        if self._payload_type != "Int":
            return (
                "<unsupported TraversalControl payload type "
                f"{self._payload_type}>"
            )
        rendered = _TopalIntPrinter(payload).to_string()
        if rendered.startswith("<"):
            return f"<invalid TraversalControl payload: {rendered}>"
        constructor = "Finish" if tag else "Continue"
        return f"{constructor} {rendered}"


class _TopalListPrinter:
    """Render an immutable topal-native List through its element type."""

    def __init__(self, value, element_type):
        self._value = value
        self._element_type = element_type

    def to_string(self):
        address = int(self._value)
        inferior = gdb.selected_inferior()
        entries = []
        visited = set()
        pair_types = ("(Int, Int)", "(Int, String)", "(String, Int)")
        indexed_entry_type = "(index : Int, value : Int)"
        node_size = (
            32
            if self._element_type == indexed_entry_type
            else 24 if self._element_type in pair_types else 16
        )
        while address:
            if address in visited:
                return "<cyclic List>"
            if len(entries) >= 100_000:
                return "<List too large to render safely>"
            visited.add(address)
            try:
                node = bytes(inferior.read_memory(address, node_size))
            except gdb.MemoryError:
                return "<unreadable List node>"
            if self._element_type == "Effect":
                payload = node[0]
                if payload:
                    return f"<invalid Effect value {payload}>"
                entries.append("Effects ()")
            elif self._element_type == "Int":
                payload = int.from_bytes(node[0:8], "little")
                if not payload:
                    return "<invalid null List Int entry>"
                rendered = _TopalIntPrinter(payload).to_string()
                if rendered.startswith("<"):
                    return f"<invalid List Int entry: {rendered}>"
                entries.append(rendered)
            elif self._element_type == "String":
                payload = int.from_bytes(node[0:8], "little")
                if not payload:
                    return "<invalid null List String entry>"
                rendered = _TopalStringPrinter(payload).to_string()
                if rendered.startswith("<"):
                    return f"<invalid List String entry: {rendered}>"
                entries.append(rendered)
            elif self._element_type == "(Int, Int)":
                left = int.from_bytes(node[0:8], "little")
                right = int.from_bytes(node[8:16], "little")
                if not left or not right:
                    return "<invalid null List (Int, Int) field>"
                left = _TopalIntPrinter(left).to_string()
                right = _TopalIntPrinter(right).to_string()
                if left.startswith("<") or right.startswith("<"):
                    return f"<invalid List (Int, Int) entry: ({left}, {right})>"
                entries.append(f"({left}, {right})")
            elif self._element_type == "(Int, String)":
                left = int.from_bytes(node[0:8], "little")
                right = int.from_bytes(node[8:16], "little")
                if not left or not right:
                    return "<invalid null List (Int, String) field>"
                left = _TopalIntPrinter(left).to_string()
                right = _TopalStringPrinter(right).to_string()
                if left.startswith("<") or right.startswith("<"):
                    return (
                        "<invalid List (Int, String) entry: "
                        f"({left}, {right})>"
                    )
                entries.append(f"({left}, {right})")
            elif self._element_type == "(String, Int)":
                left = int.from_bytes(node[0:8], "little")
                right = int.from_bytes(node[8:16], "little")
                if not left or not right:
                    return "<invalid null List (String, Int) field>"
                left = _TopalStringPrinter(left).to_string()
                right = _TopalIntPrinter(right).to_string()
                if left.startswith("<") or right.startswith("<"):
                    return (
                        "<invalid List (String, Int) entry: "
                        f"({left}, {right})>"
                    )
                entries.append(f"({left}, {right})")
            elif self._element_type in ("List (Int, String)", "List(Int, String)"):
                payload = int.from_bytes(node[0:8], "little")
                rendered = _TopalListPrinter(payload, "(Int, String)").to_string()
                if rendered.startswith("<"):
                    return f"<invalid nested List entry: {rendered}>"
                entries.append(rendered)
            elif self._element_type == indexed_entry_type:
                index = int.from_bytes(node[0:8], "little")
                value = int.from_bytes(node[8:16], "little")
                index_order = int.from_bytes(node[16:20], "little")
                value_order = int.from_bytes(node[20:24], "little")
                if not index or not value:
                    return "<invalid null List indexed entry field>"
                if (index_order, value_order) != (0, 1):
                    return "<invalid List indexed entry field order>"
                index = _TopalIntPrinter(index).to_string()
                value = _TopalIntPrinter(value).to_string()
                if index.startswith("<") or value.startswith("<"):
                    return f"<invalid List indexed entry: ({index}, {value})>"
                entries.append(f"(index is {index}, value is {value})")
            else:
                return f"<unsupported List element type {self._element_type}>"
            address = int.from_bytes(node[node_size - 8 : node_size], "little")
        rendered = "Empty"
        for entry in reversed(entries):
            rendered = f"Entry ( {entry}, {rendered} )"
        return rendered


class _TopalSequenceContainerPrinter:
    """Render an immutable compiler-private Array or Set of Int."""

    def __init__(self, value, kind):
        self._value = value
        self._kind = kind

    def to_string(self):
        address = int(self._value)
        if not address:
            return f"<invalid null {self._kind}>"
        inferior = gdb.selected_inferior()
        try:
            header = bytes(inferior.read_memory(address, 16))
        except gdb.MemoryError:
            return f"<unreadable {self._kind}>"
        count = int.from_bytes(header[0:8], "little")
        current = int.from_bytes(header[8:16], "little")
        if count > 100_000:
            return f"<{self._kind} too large to render safely>"
        entries = []
        visited = set()
        for _ in range(count):
            if not current:
                return f"<truncated {self._kind}>"
            if current in visited:
                return f"<cyclic {self._kind}>"
            visited.add(current)
            try:
                node = bytes(inferior.read_memory(current, 16))
            except gdb.MemoryError:
                return f"<unreadable {self._kind} entry>"
            payload = int.from_bytes(node[0:8], "little")
            if not payload:
                return f"<invalid null {self._kind} Int entry>"
            rendered = _TopalIntPrinter(payload).to_string()
            if rendered.startswith("<"):
                return f"<invalid {self._kind} Int entry: {rendered}>"
            entries.append(rendered)
            current = int.from_bytes(node[8:16], "little")
        if current:
            return f"<invalid {self._kind} entry count>"
        return f"{self._kind} (" + ", ".join(entries) + ")"


class _TopalBagPrinter:
    """Render an immutable compiler-private Bag of Int."""

    def __init__(self, value):
        self._value = value

    def to_string(self):
        address = int(self._value)
        if not address:
            return "<invalid null Bag>"
        inferior = gdb.selected_inferior()
        try:
            header = bytes(inferior.read_memory(address, 24))
        except gdb.MemoryError:
            return "<unreadable Bag>"
        total = int.from_bytes(header[0:8], "little")
        distinct = int.from_bytes(header[8:16], "little")
        current = int.from_bytes(header[16:24], "little")
        if distinct > 100_000 or total < distinct:
            return "<invalid Bag counts>"
        entries = []
        visited = set()
        observed_total = 0
        for _ in range(distinct):
            if not current:
                return "<truncated Bag>"
            if current in visited:
                return "<cyclic Bag>"
            visited.add(current)
            try:
                node = bytes(inferior.read_memory(current, 24))
            except gdb.MemoryError:
                return "<unreadable Bag entry>"
            payload = int.from_bytes(node[0:8], "little")
            multiplicity = int.from_bytes(node[8:16], "little")
            if not payload or not multiplicity:
                return "<invalid Bag entry>"
            rendered = _TopalIntPrinter(payload).to_string()
            if rendered.startswith("<"):
                return f"<invalid Bag Int entry: {rendered}>"
            entries.append(f"({rendered}, {multiplicity})")
            observed_total += multiplicity
            current = int.from_bytes(node[16:24], "little")
        if current or observed_total != total:
            return "<invalid Bag counts>"
        return "Bag (" + ", ".join(entries) + ")"


class _TopalMapPrinter:
    """Render an immutable compiler-private Map from String to Int."""

    def __init__(self, value):
        self._value = value

    def to_string(self):
        address = int(self._value)
        if not address:
            return "<invalid null Map>"
        inferior = gdb.selected_inferior()
        try:
            header = bytes(inferior.read_memory(address, 16))
        except gdb.MemoryError:
            return "<unreadable Map>"
        count = int.from_bytes(header[0:8], "little")
        current = int.from_bytes(header[8:16], "little")
        if count > 100_000:
            return "<Map too large to render safely>"
        entries = []
        visited = set()
        for _ in range(count):
            if not current:
                return "<truncated Map>"
            if current in visited:
                return "<cyclic Map>"
            visited.add(current)
            try:
                node = bytes(inferior.read_memory(current, 24))
            except gdb.MemoryError:
                return "<unreadable Map entry>"
            key = int.from_bytes(node[0:8], "little")
            value = int.from_bytes(node[8:16], "little")
            if not key or not value:
                return "<invalid null Map entry field>"
            key = _TopalStringPrinter(key).to_string()
            value = _TopalIntPrinter(value).to_string()
            if key.startswith("<") or value.startswith("<"):
                return f"<invalid Map entry: ({key}, {value})>"
            entries.append(f"({key}, {value})")
            current = int.from_bytes(node[16:24], "little")
        if current:
            return "<invalid Map entry count>"
        return "Map (" + ", ".join(entries) + ")"


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
    if (
        value_type == "SerializationStream"
        or storage_type == "struct TopalSerializationStreamHeader *"
    ):
        return _TopalSerializationStreamPrinter(value)
    if storage_type.startswith("struct TopalTask."):
        return _TopalTaskPrinter(value, value_type)
    if storage_type.startswith("struct TopalLocation."):
        return _TopalLocationPrinter(value, value_type)
    if value_type == "Version" or storage_type == "struct TopalVersionHeader *":
        return _TopalVersionPrinter(value)
    if storage_type.startswith("struct TopalModular."):
        return _TopalModularPrinter(value, value_type)
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
    if value_type == "Error (lang generator GeneratorErrorCode)":
        return _TopalErrorPrinter(value, "generator")
    if value_type == "lang arithmetic ArithmeticErrorCode":
        return _TopalErrorCodePrinter(value)
    if value_type == "lang generator GeneratorErrorCode":
        return _TopalGeneratorErrorCodePrinter(value)
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
    if value_type.startswith("Optional("):
        return _TopalOptionalPrinter(value, value_type[len("Optional") :])
    traversal_prefix = "TraversalControl "
    if value_type.startswith(traversal_prefix) and storage_type.startswith(
        "struct TopalTraversalControl."
    ):
        return _TopalTraversalControlPrinter(
            value, value_type[len(traversal_prefix) :]
        )
    list_prefix = "List "
    if value_type.startswith(list_prefix) and storage_type.startswith(
        "struct TopalList."
    ):
        return _TopalListPrinter(value, value_type[len(list_prefix) :])
    if value_type.startswith("List(") and storage_type.startswith(
        "struct TopalList."
    ):
        return _TopalListPrinter(value, value_type[len("List") :])
    if storage_type.startswith("struct TopalContainer."):
        if value_type.startswith("Array "):
            return _TopalSequenceContainerPrinter(value, "Array")
        if value_type.startswith("Set "):
            return _TopalSequenceContainerPrinter(value, "Set")
        if value_type.startswith("Bag "):
            return _TopalBagPrinter(value)
        if value_type.startswith("Map ") or value_type.startswith("Map("):
            return _TopalMapPrinter(value)
    prefix = "Result ("
    task_suffix = ", ())"
    if value_type.startswith(prefix) and value_type.endswith(task_suffix):
        return _TopalResultPrinter(
            value, value_type[len(prefix) : -len(task_suffix)]
        )
    suffix = ", lang arithmetic ArithmeticErrorCode)"
    if value_type.startswith(prefix) and value_type.endswith(suffix):
        return _TopalResultPrinter(value, value_type[len(prefix) : -len(suffix)])
    compact_prefix = "Result("
    if value_type.startswith(compact_prefix) and value_type.endswith(suffix):
        return _TopalResultPrinter(
            value, value_type[len(compact_prefix) : -len(suffix)]
        )
    generator_suffix = ", lang generator GeneratorErrorCode)"
    if value_type.startswith(prefix) and value_type.endswith(generator_suffix):
        return _TopalResultPrinter(
            value,
            value_type[len(prefix) : -len(generator_suffix)],
            "generator",
        )
    if storage_type.startswith("struct TopalUnion."):
        return _TopalSumPrinter(value)
    if storage_type.startswith("struct TopalVariant."):
        return _TopalSumPrinter(value)
    return None


gdb.pretty_printers.append(_lookup_topal_value)
