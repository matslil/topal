# Source documentation

Topal documentation lives beside the declaration it describes. A line beginning
with `###` is a documentation comment. Consecutive documentation-comment lines
form one documentation block; the marker and one optional following space are
not part of the text. The line ends at the ordinary source newline, like a `#`
comment:

```topal
### Return the smaller value.
### Equal values preserve the left operand.
pub min is fn (left : (Value : TotalOrder), right : Value) -> Value
  left <= right then left otherwise right
```

A block documents the next declaration or parameter in the same enclosing
declaration list. Blank lines and ordinary comments may separate it from that
target. It cannot cross a scope boundary or another documentable declaration.
Documentation does not attach to an identifier use in an expression.

Functions, generators, types, aliases, constructors, fields, parameters,
capabilities, effects, and other named declarations may be documented. An
operator is documented by the function declaration which gives that operator
its stable identity. Each overload has its own documentation even when tools
group overloads under one displayed name.

Parameter documentation belongs immediately before the parameter. Parameters
normally need no documentation when their name and type already explain their
role:

```topal
### Find an entry satisfying `predicate`.
pub find is fn (
  values : List (Value : Type),
  ### Called from left to right and no longer called after a match.
  predicate : fn (Value) -> Boolean
) -> Optional Value
```

Documentation is plain Unicode prose rather than embedded output-format markup.
Tools preserve its paragraph breaks and may recognize declaration names for
cross-references. Declaration introspection exposes the normalized text through
`lang DeclarationView.documentation`; aliases retain their own documentation.
Built-in declarations carry the same metadata as source declarations.

## Reference generation

The Topal documentation tool generates reStructuredText reference files for
the source paths explicitly given to it. A file selects only that file. A
directory selects its Topal files directly within that directory; `--recurse`
also visits descendant directories. No standard library or other implicit
source path is added. Documenting `std` therefore requires explicitly passing
the standard-library path.

Built-in declarations are excluded unless the built-in `lang` namespace is
requested with the tool's built-in inclusion option. Generated entries contain
the qualified name, Topal signature or declaration syntax, overload-specific
description, documented parameters, and any recorded limitations or common
gotchas.

## Interactive help

The source debugger's `help` command without an argument lists debugger
commands. `help name` resolves a visible declaration and prints its signature
and documentation. A qualified name disambiguates declarations; an ambiguous
unqualified name lists candidates. Built-in declarations are available through
qualified `lang` names without importing them into the debugged program.
