======================
Core Language
======================

Primitive types
===============

Primitive type is a type that is defined by the language. All other types are derived from the primitive types.

Value
- Data
  - Number
    - Real
      - Float
      - Fraction
    - Integer
      - Signed integer
      - Unsigned integer
  - Character
    - ASCII
    - Unicode
  - Boolean
- Set
- Dictionary
  - Unique dictionary
- List
  - Finite list
  - Infinite list
- Enum
- Identifier
  - Private
  - Public
  - Imported
- Accessability
  - Read-only
  - Read/write
  - Write-only
  - None
- Storage
  - Alignment
  - Size (bits/bytes)
- Pattern
- Operation
- Scope
  - Contextual scope
  - Thread scope
- Constraint
  - Unit
  - Valid value range
  - Valid size range
- Lifetime
- Dependency
  - Is-a (compatible with)
  - Derived from
- State
  - Unassigned
  - Assigned


Properties
----------

Commutative
   States that in which order the terms for an operator does not change the result.
   Example: x + y = y + x

Associative
   States that the order of evaluation for a sequence of the operator does not change the result, given that the order of the terms are kept.
   Example: x + (y + z) = (x + y) + z

Distributive

Symmetry

Compound types
==============

Examples of types built from primitive types:

struct: Fixed sized dictionary

array: Dictionary with unsigned integer as key type

Constrant vs trait vs inheritance?

Operation
=========

An operation operates within a scope. It can access the scope using `self` identifier. Immediately to the right there is an expected argument, accessible using `arg` identifier.

If `arg` is a list it can also accept single items as this is seen as a list of length one. If `arg` is given several times, all occasions are merged. For list this is done by appending the list, for dictionary it fills in different entries of the dictionary. Note that if the dictionary does not accept duplicates, a specific key may only be given once.

Set
---

Union (∪)
   Returns all elements belonging to either set.

Intersection (∩)
   Returns elements belonging to both sets.

Difference (-)
   Returns elements belonging to the left set but excluding elements belonging to right set.

Complement (')
   Returns all elements not belong to set.

Disjoint
   Whether sets has no common element.

Proper subset ()
   Whether all elements in left list belongs to right list.

Category theory
---------------

Category theory is about morphisms between objects. Objects could be finite sets containing elements, in which case, morphism describes how elements in one object transforms into objects in another object.

Follows (∘)
   Composing morphisms. Having morphisms f and g, where f morph A to B, and g morph B to C, you have g ∘ f (note the order). This is called composition.
   Composition translates to programming like g(f(A)).

For programming (category of programming) object can be seen as types (not 100% accurate, but close enough), and morphism as functions.

Identity
   Morphism from object A to object A, i.e. to itself, doing nothing.
   Similar to 0 in mathematics, since category theory is about what morphisms does.

Associative
   Composition must be associative, i.e. the order we evaluate morphisms shouldn't matter given that the morphism order stays the same.
   Given morphisms f, g, h: h ∘ g ∘ f = (h ∘ g) ∘ f = h ∘ (g ∘ f)

Isomorphic
   Objects A and B are isomorphic if f: A->B and g: B->A, f ∘ g = I(B) and g ∘ f = I(A).
   I.e., f and g are doing the same kind of morphism, just in different direction.

Cross product
   A x B is:
   f: A x B -> A, g: A x B -> B, h: C -> A, i: C -> B

    A <- f -- A x B -- g -> B
    ^           ^           ^
     \          |          /
      f         h         g
       \        |        /
        \       |       /
         ------ C -----

Currying
   Transform a function taking multiple arguments into a function taking a single argument producing a new function taking next argument and so on.
   f:(X x Y) -> Z, currying produces h: X -> (Y -> Z)
   Requires the category to be closed and monoid.

Monoid
   Takes two objects of a certain type and produces a new object of same type. It works like an accumulator, and needs to handle the case when one of the types to be combined is a null object.

Nominal data
   Cannot be ordered or compared (other than for equality).

Ordinal data
   Has an order, you can order it and compare which is greater than the other.

Mapping category theory to programming language type system:

===============          ============
Category theory          Programming
===============          ============
object                   type
morphism                 function
functor                  polymorphic type
natural transformation   polymorphic function

Mapping category theory into programming language
-------------------------------------------------

Identify function
   Needs a do-nothing function, or identity function.

Unit data type
   Need a 1 constant, a unit type.

In and out types
   Composition has input and output types, no side effects.

Category properties
-------------------

Identity

Inverse
  g:B->A is an inverse for f:A->B if:
    f ◦ g = IB (identify of B) and g ◦ f = IA (identify of A).

Isomorphism
   If A can be morphed to B, and B to A, then A and B are isomorphic.
   The morphisms f:A->B and g:B->A are isomorphisms.
   f is the inverse of g, and g is the inverse of f.
   "Really the same", in category terms.

Adjoint functors
   Weaker than isomorphism.

Monomorphism
   f ◦ g = f ◦ h then g = h

Epimorphism
   There is a mapping from A to B for every element in B.
   Function f: A->B is onto if for every y in B there is an x in A with y=f(x)
   g ◦ f = h ◦ f then g = h

Functors
   Used when objects in a category are categories in themselves.
   Structure-preserving maps between categories

Universal constructions: Units, voids, products, sums, exponentials

Cartesian closed categories
   Morphisms behave like real functions, there is currying and applying a curried morphism.

Topoi
   A cartesian closed category with more axioms that makes object behave like sets, in particular for each object there exists an object of its 'subobjects'.

Discrete category
   Set: Elements are unrelated to each other. Only identify morphisms.

Category of algebras
   Each object is a sort, with a binary function over that sort
   Each morphism is a translation from one algebra to another, preserving structure

Category if temporal logic
   Modular specifications and decompose system properties across them

Automata theory

Logic as a category

The category of logics

List
   Elements have an order, i.e. has a previous and next. Does this imply there is an equal operator?

Universal property
   If two objects has the same universal property, they are said to be isomorphic (i.e. the same kind of stuff).

Domain and codomain
   For morphism f: A->B, A is the domain, B is the codomain.

Universal constructs
   General properties that apply to all objects in a category
   Each construct has a dual, formed by reversing the morphisms
   Examples:
   - initial and terminal objects
   - pushouts and pullbacks
   - colimits and limits
   - co-completeness and completeness

Initial objects
   S is initial object if for every object X there is exactly one morphism f:S->X
   If S1 and S2 are both initial objects, then there is an isomorphic function between them

Terminal objects
   T is a terminal object if for every object X there is exactly one morphism f:X->T
   If T1 and T2 are both terminal objects, then there is an isomorphic function between them


Categories
----------

Modularity
   - Decompose into processes/threads
   - Decompose into source code components
   - Decompose into different use cases/requirements
   Goal
   - Information hiding
   - Compositional verification
   - Compositional refinement
   Building blocks
   - Modules
     - Interface
     - Structure
     - Behavior
   - Module interconnections
   - Operations on modules (e.g. compose two modules to form a third)

Algebra

Logic

State machine

Events

Messaging

Patterns

Containment

Relations (?)

Properties
----------

Has initial

Has final

Has next

Has prev

Has index

Can be appended

All elements in object are unique

List
----

VALUE .. VALUE -> LIST
   Returns a list of values from and including left VALUE to and excluding right VALUE.
   If left VALUE is none, start from first possible VALUE.
   If right VALUE is none, never stop.

VALUE ..= VALUE -> LIST
   Returns a list of values from and including left VALUE to and including right VALUE.

LIST every UNSIGNED -> LIST
   Returns every UNSIGNED entry of LIST.

TYPE repeat VALUE -> LIST
   Repeat VALUE forever, creating an infinite list.

LIST fold VALUE, FUNCTION -> VALUE
   Start with left VALUE, for each LIST entry, call FUNCTION with accumulated value as first parameter and LIST entry as second parameter. Evaluates to a VALUE.

LIST take UNSIGNED

Math
----

Structure
---------

BOOLEAN then VALUE -> (VALUE or none)
   If left side is true, return VALUE else none.

(TYPE or none) else VALUE -> (VALUE or none)
   If left side is none, then return VALUE, else return none.

SCOPE yield VALUE
   Returns VALUE but execution continues with next expression.

SCOPE return VALUE
   Returns VALUE and exit SCOPE.

LIST foreach FUNCTION -> VALUE
   Invokes FUNCTION for each entry in LIST setting function arg to the entry value. Returns value(s) yielded or returned by function. If function uses return, the foreach breaks. For yield, it will produce a list as return value where each yield produces one entry.

FUNCTION : VALUE -> VALUE
   Invokes FUNCTION with VALUE as arg. VALUE is defined by yield and/or return, or none if neither is used.

VALUE is VALUE -> SCOPE
   Defines an alias. Can be used to define an identifier, assign a variable, rename an (imported) scope.
   Returns the scope the alias lives in.

VALUE = VALUE -> BOOLEAN
VALUE > VALUE -> BOOLEAN
VALUE < VALUE -> BOOLEAN
VALUE >= VALUE -> BOOLEAN
VALUE <= VALUE -> BOOLEAN
VALUE <> VALUE -> BOOLEAN
   true if VALUE is equal, greater than, less than, greater than equal, less than equal, or not equal to VALUE, false otherwise.

VALUE in RANGE -> BOOLEAN
   true if VALUE is in RANGE, false otherwise.

x foreach ( 0 .. none every 2 )
    print ( ("x is ") x )

SCOPE use URL -> SCOPE

