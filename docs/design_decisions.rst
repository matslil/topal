================
Design Decisions
================

.. uml::

   Alice -> Bob: Hi!
   Alice <- Bob: How are you?


This document tries to answer the question why the language is designed the way it is.

Problems to be solved
=====================

Under all the years I've been programming there are some problems that contemporary languages seems to have a hard time to solve. Like:

1. Dependencies
2. Resource ownership
3. Interfacing modules from other languages
4. Easy to find errors
5. Support multiple applications

Dependencies
   With proper dependency tracking it would be easy to code error messages that could explain what input that would have an impact on the error. It would also be easy to implement a reverse runtime debugger, since you would only need to store events coming from outside the module being debugged.
   It would also make it easier to make proper module management with automatic semantic versioning where it would be possible to know by looking at the code what change is a breaking change, what change introduce a feature and what change is merely a bug fix.
   Automatic dependency resolution to retrieve needed modules to run application would also be simplified.

Resource ownership
   Similar concept as Rust is using, but make better use of inference so it becomes more developer friendly. No need to explicitly deallocate resource, no possibilities to reference non-existing resources.

Interfacing modules from other languages
   It is common to call other applications from within the current application. Doing so should be similar to calling a local function. Same reasoning for calling functions in compiled shared libraries or running Java code.

Easy to find errors
   Coding and design errors should be signalled as soon as possible, preferably during editing. Having a strict typing system combined with type inference, should make it possible to both ensure correct code without having to write loads of boiler plate code.

Support multiple applications
   Most generic programming languages tend to work best for a certain type of application. Some are better at describing a user interface, some are better at event handling, some at handling low level access with hardware interrupts and so on.
   Sisdel intends to build on a simple core which then can be extended in different directions depending on use case. This way you can have the advantages that domain specific programming languages give you, without having to learn a whole new language.

Goals
=====

1. Well defined language, no undefined nor implementation defined behavior
2. Robust, logic errors are detected as early as possible, preferably at compile time
3. Modularized, errors in one module does not propagate into another
4. Composable, modules should never be more specific than needed
5. Error indications pointing at root causes, not just symptoms

Well defined language
---------------------

Code written should have a non-ambiguous translation into machine behavior. There might be fancy things happening for sake of performance, but this must never cause the code to behave differently except for the needed resources like time and memory.

The code should also have a consistent syntax, using a few base concepts that are applied to build more complex constructs. This should make the language easier to learn and easier to port between platforms.

Robust
------

- Handle resource ownership, so compiler can make sure no resource leaks occur
- Handle resource access, so compiler can make sure no race conditions occur

Composable
-----------

- Be able to build big modules out of smaller modules without any changes in behavior of the small modules
- Be able to write knew variants of a module without needing to recompile the other parts of the code
- Simple syntax to promote using the language as a domain specific language (easier to modify its looks)

Error localization
------------------

Using the dependencies that the language provides, make an error localization that can point out the cause of the error, rather than just how they manifest themselves.

Source of Inspiration
=====================

Paradigms
---------

As compilers get access to more and more computational power, they have been able to help the developer more. This means that some details of how the program works could be removed from the language and let the compiler handle it instead.
Sisdel will build on these and add one of its own: Removal of the generic loops.

Structured programming
   Removed possibility to explicitly transfer flow of control to some other part of the code, typically using the "goto" statement. The structured programming paradigms said that such explicit transfer of control should be done using structures like conditionals and loops.
   **Sisdel**: There is no goto, but there are conditionals and iterators. There is also an at exit handler.

Functional programming
   Removed possibility for having variable assignment, only initializations were allowed. This removed a lot of robustness problems around illegal states due to parts of an object state to be modified but missed to update other parts of the state.
   **Sisdel**: Assignment creates new instances, not modifying them. Mutability will be simulated where a new object with the same name and scope is created, but where an implicit generation counter or hash value is used to make each instance unique.

Object-oriented programming
   Removed the possibility to do indirect transfer of control directly, typically using function pointers. Instead this is done using polymorphism or interface.
   **Sisdel**: Sisdel has objects but no classes, instead objects are created by object factories. It is possible in Sisdel to declare object compatibility, where an object can be declared as being compatible with some other. This simulates inheritance on the interface level, but is implemented using containment.

Removal of generic loops
   A generic loop is hard to understand, hard to know if it terminates when it should and hard to know whether there is a risk of running it one time too many or too few. It is much easier to understand loops that iterates a list, a structure or that are designed to loop forever.
   **Sisdel**: Sisdel supports forever loops and iterating lists.

Resource allocation is initialization
   Make initialization and allocation of a resource an atomic operation. Avoids all pitfalls when handling an allocated but uninitialized resource.
   **Sisdel**: Dependency tracking is used to make sure no code path can use an uninitialized resource.

Atomicity, consistency, isolation, durability
  This is typical requirement for a database operation. If you modify your database, you want this modification to either success or fail (atomicity), to never set the database in an unconsistent state, be unaffected by other operations done in parallell, and if successfull be stored in a way that ensures that it is not lost by mistake.
  **Sisdel**: Operations on individual objects are ensured by the compiler to be compliant with ACID in the scope of the running program. Possibility to describe dependencies between objects, which will make the compiler to ensure this for more complex scenarios.

SOLID
  This is for interface design, where S is for single responsibility, i.e. the interface should only do one thing and not a mix of unrelated things. O is for opened for extensions, but closed for changes. You can add to the interface, but if you need to change it, you need to design a new interface. This ensures backwards compatibility. L is for Liskov substitution, which means that if an object S inherits form object T, then object S can be used where T is expected without breaking any desirable properties of the program. I is for interface segration, which means that a user of one interface must not be forced to be dependent on other interfaces they do not use.D is for depency inversion, which means that higher-level functionality must not be dependent on low-level functionality. This means that low-level functionality must not be directly accessible from high-level functionality.
  **Sisdel**: *S*: Single responsibility is something the programmer must ensure, it is hard for the compiler to know this. *O*: Sisdel objects cannot be modified once defined, but you can define a new object which extends an existing object. *L*: If an object is declared as compatible with another object, Sisdel will ensure that this is true from a programming language perspective. *I*: Objects in Sisdel are not allowed to force user of the object to be dependent on some other object which the user of the object might not want. *D*: Since the definition of higher and lower level of functionality is unknown to Sisdel it cannot ensure higher level functionality does not expose lower level functionality.

Tiers of programming language design
------------------------------------

This tries to describe how high-level the programming language is:

0. Chaos, i.e. nothing understandable
1. Single opaque object, example: Calculator
2. Patterns, example: Assembler
3. Hierarchies, example: C
4. Black boxes, example: C++, Haskell

The lowest level of entity in Sisdel is the object, which could resemble the single opaque object. It has an interface, like a calculator, but expose nothing about how it works.

Sisdel uses types to be able to describe patterns, similar to what other languages do. However, Sisdel see types as a type itself, allowing things like iterators and conditionals to work on types as well. This makes it possible to have very expressive patterns.

Sisdel defines dependencies to describe hierarchies. Object-oriented languages typically use containment and inheritance for this. In Sisdel, those are translated to dependencies, together with other types of dependencies, e.g. required ordering. Since dependencies is an integral part of the language, things like iterators and conditionals can be used on them as well. This allows an object to contain another object given some condition, as long as it can be determined compile time.

Modes of programming
--------------------

Events
   Wait for events and perform an action depending on event. Events can be sent, received and broadcasted to all or to specific groups.
   **Sisdel**: Sisdel has a scope which can tell who called the object method even for remote calls, which makes this useful for event indication. Sisdel has also a thread concept, and combined with lazy evaluation this mimics the behavior for asynchronous sending of events. Broadcasts could be mimiced by having a broadcast receive lists in the shared scope object.

Pipelines
   Data progresses in a pipeline fashion, where output from one stage is the input for the next. Pipelines can be forked and merged. Broadcast status messages can be sent to the whole pipeline or a part of it. Exceptions will terminate the whole pipeline.
   **Sisdel**: This is the default behavior for object method calls, since output of a one call is the input for the next. Broadcasts can be mimiced using broadcast receiver lists in the scope object. Sisdel support exceptions.

Descriptive
   Describes what to be done, not how. Example would be make files describing how the files depend on each other, but does not describe exactly which order to build them.
   **Sisdel**: This is the default behavior of Sisdel, since it defaults to functional behavior with implicit as well as explicit dependency support.

Common problems
---------------

Module and language versioning
   See for example the mess with Python versions, where virtual environments are needed in order to handle the version problem. Upgrading the system wide Python interpreter is not for the faint hearted.

Snapshots
   Git has a good concept of snapshots, this is also reinvented in other tools like Docker, Vagrant, etc. Instead of constantly reinventing this wheel, the programming language itself should have a support for doing a snapshot of the current state, and have mechanisms for determining what should be included in such a snapshot.

Meta-data sharing
   Solving above problems for just one language is not enough. The programming language should have support for understanding standard ways other languages use for describing different kinds of meta-data. As for versioning, this could be being co-operative with the system package manager, Python pip, etc. So whenever there is a dependency on modules written in other languages, meta-data support for describing what version, variant etc is needed should also be included.

Configuration storage
   How to (persistently) store configuration should not be a concern for the application needing to store its configurations. There should be a standard API defined by the programming language how this should look like. An adapter component could then define how a set of components store their configurations. How configuration is stored depends on how and where the component is deployed. In docker containers, there might be one preferred way, for a Linux system another, for Windows a third, if deployment is system wide this might be different compared to if it is for a specific user only, if using cloud micro-service architecture there might be yet another preferred way, and so on.

Domain-Specific Languages
-------------------------

Looking at domains-specific languages should give inspiration into what syntax fits a specific application, and also what kind of abstractions that are most useful. The end goal is to have a language which can have different incarnations suiting different problem domains. This way, programmers do not need to learn a completely new language to be able to write code for a specific problem domain, they only need to learn a new area of Sisdel.

The following problem domains were considered:

- Text search
  - Regular expressions
- Text type setting
  - LaTeX
- Database queries
  - SQL
  - GraphQL
- Relational diagrams, e.g. state machines, transaction diagrams, dependency diagrams
  - Graphviz
  - PlantUML
- Protocol/interface description
  - CORBA
- Data sequencing
  - YAML
  - JSON
  - XML
  - Google's Protocol Buffers
- Hardware description
  - SpinalHDL
  - VHDL
  - Verilog
- Build description
  - Make
  - CMake
  - Rust Cargo
- Sound creation
  - CSound
  - SuperCollider
  - Chuck
- Syntax description
  - Backus-Naur Format

### Text search

Regular expressions are among the most used for text search. It has some advantages:

- Can search any text
- Commonly used

It has also some disadvantages:

- Hard to read, especially for complex search patterns
- Has no concept of scope, e.g. search for a word, paragraph, path entry, etc

The disadvantages can be solved by:

1. Make literals more explicit, so no escapes are needed for characters with special meanings
2. Make it possible to define context, and have operators that can make use of that (e.g. word, path entry, etc)

Examples:

    search-for <- match text ignore-case '(prefix: )' ? int ( '(,)' int ) * '(: )' word nl

This creates a match expression similar to how regular expressions work. It will match when a string is found which optionally begins with the string literal 'Prefix: ', followed by on or more integers separated with literal ',', followed by literal string ': ', followed by a word and ending with a platform dependent new-line character sequence. The matches are done with character case ignored.

Building blocks
===============

Sisdel is a class-less object oriented language. Objects are values with some meta-data associated with it. In some cases these values are available directly, e.g. strings and numbers, and sometimes they need to be calculated, e.g. object methods.

Error handling is done primarily using error return values. Exceptions are only used to describe when an object has been compromised. When an object throws an exception, it signals that it can no longer be used. Any further attempts to use the object will result in an exception. In this case, the only use for the object is to have a context for the exception.

Object

Type

Operator

Type
----

Type in Sisdel consists of the following parts:

1. Fundamental types
2. Constraints
4. Representation

Fundamental type are meant to describe fundamentally different things, while constrictions are meant to limit the use of the type. Side effects are operations done to a device, and signals synchronization points. The representation is how the data is stored, and does not by itself prohibit use but rather triggers conversions.

Fundamental types
~~~~~~~~~~~~~~~~~

Fundamental types describes things which cannot be used interchangably without conversion.

Number
   Any number, rational, irrational, complex.

Boolean
   Either true or false.

Comparison
   One of: less, equal, greater.
   Result from the <=> operator.

Character
   UTF-8 Unicode encoded character.

Set
   Collection of objects with no ordering.

Map
   Collection of key value pairs. Different keys can have different types, same is true for values. Key cannot be a container.

Identifier
   Name that only has a meaning for the compiler, and is not associated with a specific value. Typically used to address objects so they can be referred to in the code.

Operator
   List of expressions executed when operator is used. The expression will have a variable self which is the object left to operator, and arg which is the object to the right of operator. There is also a thread object representing execution environment for the operator.

.. NOTE::
   Set and map has a size, and this size can be infinite. A random generator method would be an example of something that returns an infinite list. You cannot freely mix inifinite lists with finite lists, you need to specify a portion of the infinite list to do a combination.

The following groups of fundamental types exists:

Value
   Includes number, boolean and character.

Container
   Include set and map.

Some types provided by Sisdel built on fundamental types:

List
   Ordered set.

String
   List of characters.

Stream
   Serial list.

Constraints
~~~~~~~~~~~

Constraints can be put on types to limit what is accepted. A constraint expression is basically an object method applied to one type with the other type as parameter, and if this expression returns true, those two types are compatible.

Constraint expressions can work on meta-data to restrict number of elements in an array, whether all elements must have same type, specify accepted units and restrict value representations. Constraint expressions can also work on value to restrict value range or precision.

As a special case there are units. Unit has as its sole purpose to create incompatible types, and is typically used to indicate types that are not interchangable even though Sisdel type inference would accept them. This is useful for example to distinguish two integers where one might be weight and the other length. These are very different things, but since both are integers they could be used interchangably and therefore potentially cause bugs. Assigning different units to them makes them non-compatible, and makes it illegal to specify length when weight was expected.

There is also a state concept which can be used by constraints. State is another meta-data associated with objects.

Types of constraints:

Unit
   Applies to: Value.
   Unit is used to make custom types and be able to describe compatibility between them. It is also possible to specify for each operation what type is returned by the operation given what type(s) are given as input.

Valid values
   Applies to: Any.
   Set of value values.

Size
   Applies to: Container.
   Size can be finite or infinite, which makes distinct types. In order to use infinite container where a finite is expected, you must specify how much of the inifite container to use.
   Size can be set compile time or at execution time.

Serial
   Applies to: Any.
   Whether reads/writes to and from the container matters. For example, if using a map and do reads and writes to different elements in the map, those reads and writes will be performed in the exact order as issued. This is useful when describing interactions going outside of the Sisdel domain, for example when accessing hardware registers or using remote protocols.

Ordered
   Applies to: Any.
   This constraint is set implicitly on any object that has an <=> operator.
   Can be set explicitly on objects without <=> operator, in which case the order will be defined by the order elements are inserted.

Element
   Applies to: Container.
   Type constraint applied to every element within the container.

Duplicates
   Applies to: Container.
   Allow container to have several occurrences of the same object, or in the case of map, for the same key. For ordered containers these entries will be kept in insert order.
   The default behavior for sets is to ignore duplicates, i.e. attempt to insert an already existing element will simply be ignored. For maps, attempts to insert for the same key will result in the value being replaced with the new value given.

Compatibility
   Applies to: All.
   Type constraint applied to the object to ensure this object is always type compatible with the given object.

Derived from
   Applies to: All.
   Which objects that influenced what content this object has. Must be complete, i.e. there should not be more or less objects. This constraint is usually inferred by the compiler whenever a new object is created, but can be explicitly enforced to constrain a type.

Commutative
   Applies to: Operator.
   If left-hand side of the operator is swtiched with the right-hand side, the result is the same.

Associative
   Applies to: Operator.
   If operator is applied multiple times in a row in an expression, placing parenthesis will not change the result.

Representation
~~~~~~~~~~~~~~

Representation describes how the value is stored, e.g. number of bits used, endian, data format. It can for example be used to say that a map is stored as Yaml. If a specific representation is requested, and the value has another representation, this triggers a conversion. This is an operator run on the original representation whose return value need to be of the expected representation. If no such conversion has been defined, this becomes a type incompatibility error.

- Storage size in bits or bytes
- Encoding (e.g. IEEE 754, UCS-2, UTF-32, ... How to handle home-made formats?)
- Memory address location

Using the language
------------------

Working with hardware
~~~~~~~~~~~~~~~~~~~~~

If your hardware defines a register with 32 individual bits, where reading and/or writing to them causes side effects, you could define it like the following:

reg is list as
    stream                 # says access order to the stream matters
    ordered                # ordered list becomes array
    element boolean        # each value can only have values true or false
    element storage-size 1 as bits # each element only occupies one bit of storage space
    size 32                # number of elements in array
    address hex 8000'fe00  # memory address mapped for this array

How to make sure individual bits are accessed as they should would depend on hardware description used for the Sisdel compilation. For architectures support addressing individual bits this will be used, others might support reading the register, modify the bits being affected, and write the result back, and yet others might need a shadow register to avoid having to read current value.

Describing sequences
~~~~~~~~~~~~~~~~~~~~

Examples of where sequences can be useful would include describing data encoding, message API or pattern matching.

Example::

    my-sequence is list unsigned #( message version )# ( unsigned as nr-entries ) list string as ( size nr-entries )

This defines a type of name my-sequence that starts with an unsigned number, which an inline comment explains is the version number, followed by another unsigned number which is associated with the name nr-entries, followed by a list of strings, where the size of the list is determined by nr-entries.

If this is to be used to define a message format to be used externally, this needs to be serialized, or encoded, into a format suitable to be transmitted. It then needs to be deserialized, or decoded, to an object Sisdel understands.

One common encoding format used for configuration files and REST HTTP APIs is YAML. The Sisdel yaml type can be map (object in YAML), list, integer or string. These can be combined. Since my-sequence above fits this, YAML code be used like this::

    message as my-sequence is ( 1 , 2 , '(Hello)' , '(world)' )
    print message as yaml

This would print the following::

    [1,2,['Hello','world']]

Type compatibility
~~~~~~~~~~~~~~~~~~

Object methods are not a type in themselves. Object method types are equivalent with their return type if the method takes no parameters. If the method takes parameter it is type equivalent with a map where the type of the values are the return type of the method, and the type of the key is the type of the parameter.

This means that any context requiring a simple value can be replaced with an object method returning same type of value, and also vice versa.

Similarly, any context requiring a map can be replaced with a method whose return type matches the map value type and method parameter type matches map key type. Since map key type can be different for different keys, any valid key type for the map must match all valid types for the object method return value, and same is true for map value type and object method parameter type.

.. NOTE::
   Side-effects are part of the type. Since immediate values and maps cannot have side-effects, they will never be type compatible with object methods having side-effects.

An array is a map where the key type is constrained to be unsigned integer. This means that an object method taking unsigned integer parameter is type compatible with array, if the array elements are type compatible with the return type of the object method.

As a special case, an array or map with single value is type compatible with each other or an immediate value if the values themselves are type compatible. An array storing a single string, or a map storing a single string as value, or an immediate value being a string, are all type compatible and can therefore all be used interchangably.

State is not by itself a type, but can be used with constrictions to describe a type. The state needs a context to have a meaning, which also mean that different contexts can have same name of state, but refer to different things.

Parsing
-------

Tokens
~~~~~~

Each token is separated by white space. The only characters not allowed for tokens are control and white space characters. Every token must be separated by white space.

Some characters have special meaning when parsed. For parenthesis characters, ({[, any character surrounding them must match the matching parenthesis character, )}], in reverse order. So the token --( is matched by )--, while {{ is matched by }}. Any token within such pair are being grouped.

Indentation
~~~~~~~~~~~

Each line can start with zero or more tab character. This is the only valid place for tab characters. Each tab character represents one indentation level. All consecutive lines with the same indentation represents item in a set, and is therefore equivalent to separating them with comma character. I.e., the following::

    mylist is
        1
        2
        3

is equivalent with::

    mylist is ( 1 . 2 . 3 )

Note the space before and after comma characters, since each token must be separated by white space.

In case the indented line starts with an operator, the scope for the operator, self or left-hand side, will be where the previously less indented line left of at. It would be like the line was continued with the indented line. This can be used to break up long lines, but also to write several operation done from the same scope by having several indented lines starting with an operator.

Grouping operator
~~~~~~~~~~~~~~~~~

There is a special operator that group objects rather than operate on them. The grouping operator can also specify conditions for the objects contained, e.g. what type the scope in the group has.

A group operator starts with an operator name which has one or more of the following characters included::

    (
    {
    [

This character can be surrounded with other characters that are allowed for identifier names. The group ends when a reversed version of the start is used. The reverse is here defined as using the closing version of the parenthisis above, i.e. ) when starting with (, } when starting with { and so on. Furthermore, characters surrounding the start token must be reversed.

Here are some examples of group start and group end pairs, with no objects contained::

    ( )
    { }
    [ ]
    {{ }}
    --[ ]--
    my( )ym

Non-greedy token matching
~~~~~~~~~~~~~~~~~~~~~~~~~

Operator takes one argument, and the match is done in a non-greedy fashion. To supply several items for the operator, these items need to be contained in a group, e.g. set. Example::

    print '(Hello World)'

Here the operator print is supplied with a list of characters using '( )' grouping operator.

Note that if an operator is used to supply argument to another operator, grouping will be needed. The following will most likely not do what you intended::

    append-world is operator arg '( World)'

This would be equivalent to::

    ( append-world is operator arg ) '( World)'

And this would not even compile. You need to write this as:

    append-world is ( operator arg '( World)' )

Operator precedence
~~~~~~~~~~~~~~~~~~~

The operator precedence is very simple. The code line is simply scanned from left to right, and evaluates operators in that order. The only way to change this order is by using grouping operators.

Special values
--------------

The following special values exists:

nil
   Represents an empty set.

any
   Represents a universal set, i.e. everything.

err
   Represents an error. Map with information about the error.

true
   Boolean representing true value.

false
   Boolean representing false value.

less
   Comparison value representing sorts less than.

equal
   Comparison value representing sorts equal to.

greater
   Comparison value representing sorts greater than.

_
   Underscore represents a space character. Useful when building strings with spaces in them.

Scope
-----

Referring an object defines a scope. Indented code block also defines a scope. Some objects are created implicitly, e.g. a source file, but most objects are created in code.

Everything described in a scope is by default expected to be a complete description of something, could be a description of how a new object is created or what an operation does. All code within the scope can be executed in any order, with one exception: List. List is used to enforce explicit ordering, and can be used to describe cases when order matters. This could for example be a random number generator, which would return an infinite list. The order this list is read matters. Or it could be piece of hardware, where the order which certain registers are read or written to matters.

Object created in a scope can be modified within the scope, but cannot be used since it will not be seen as fully initialized until the scope exits. The scope would typically return the initialized object as its value, and once this has been done this object can no longer be modified. However, new objects can be created based of the original object.

Each new line restarts the scope to the containing scope. In a source file, each line written without indentation would then use the source file scope as the starting scope. Each object addressed on the line will change the scope as the line progress, until a newline character is encountered.

For indented lines, each new line starts with the scope of previous one less indented line.

A scope can also have an at-exit handler associated with it. It contains code that will be executed right before the scope is exited. This can be attached to objects to emulate destructors typically found in object-oriented languages.

Thread
------

Thread is an object containing a shared state description for an execution. Each time a new execution thread is created, e.g. when issuing an operator, a clone of the calling thread object is done and then used in the called thread scope. This works similarly to how environment variables work in Unix.

Arg
---

This object contains the right hand side of the operator arguments. The left hand side argument is inherited into current scope and can be accessed without scope operators.

Space operator
--------------

For a sequence of objects, the following happens: (matches are tried in the order listed)

MAP ANY
   Index MAP using ANY as key.

SET STE
   Merge two sets into one.

TYPE ANY
   Creates an object of type TYPE with value ANY. Note that this becomes an anonymous object in current scope.

Set
---

Set can be given in code in two different ways. Either using , character, like::

    my_set is ( 1 , 2 , 3 )

Or, using indented lines::

    my_list is
        1
        2
        3

These two gives identical results.

When using an operator that normally do not expect its right-hand argument to be a set, but is given a set, it will be applied repeatedly and return a list of result. Like::

    sum_set is 2 + ( 1 , 2 , 3 )

will set sum_set to value ( 3 , 4 , 5 ). This can be used generically, e.g.::

    fun_list is 100
        > 50 then '( large )'
        = 50 then '( medium )'
        < 50 then '( small )'
        & 1 = 0 then '( even )'

will set fun_list to ( '( largs )' , '( even )' )

Built-in operators
------------------

Conditional
~~~~~~~~~~~

BOOLEAN then EXPRESSION
   Works like an if statement. If BOOLEAN is true, then EXPRESSION is executed and value of EXPRESSION is returned. Otherwise, nil is returned.

BOOLEAN else EXPRESSION
   Works like an if-else statement. If BOOLEAN is false, then EXPRESSION is executed and value of EXPRESION is returned. Otherwise, nil is returned.

Assignment
~~~~~~~~~~

IDENTIFIER is ANY
   Define a new identifier IDENTIFIER to be associated with ANY.

ANY as CONSTRAINT
   Puts CONSTRAINT on ANY.

operator EXPRESSION
   Defines an anonymous operator which evaluates EXPRESSION. The special variable arg is defined in the scope of EXPRESSION containing the argument to the operator.

unsigned is ( number as ( 0 .. nil ) )

Arithmetic operators
~~~~~~~~~~~~~~~~~~~~

NUMBER + NUMBER
   Arithmetic addition of two numbers.

NUMBER - NUMBER
   Arithmetic subtraction of two numbers.

NUMBER * NUMBER
   Arithmetic multiplication of two numbers.

NUMBER / NUMBER
   Arithmetic division of two numbers.

NUMBER % NUMBER
   Remainder if left-hand side is divided with right-hand side.

NUMBER ^ NUMBER
   Left-hand side raised to right-hand side.

Bit-wise operators
~~~~~~~~~~~~~~~~~~

UNSIGNED & UNSIGNED
   Bit-wise and operation.

UNSIGNED | UNSIGNED
   Bit-wise or operation.

UNSIGNED || UNSIGNED
   Bitwise xor operation.

~ UNSINGED
   Bit-wise negate operation.

Container operators
~~~~~~~~~~~~~~~~~~~

VALUE select LIST
   Each element of LIST is "OPERATOR VALUE then EXPRESSION", where first VALUE is used as left-hand side of OPERATOR.
   Return value is the EXPRESSION for the first entry in LIST where "VALUE OPERATOR VALUE" evaluates to true.
   As a special case, "OPERATOR VALUE then" can be replaced with "otherwise".
   Example::

   myname select
       = '(adam)' then print '(male)'
       = '(eva)'  then print '(female)'
       otherwise print '(unknown sex)'

first LIST
   Returns first item in LIST.

last LIST
   Returns last item in LIST.

LIST zip LIST
   Returns map with left-hand side as list of keywords and right-hand side as a list of values to be associated with the keywords. Both lists need to be of same size.

CONTAINER + CONTAINER
   Appends two containers. If any CONTAINER is ordered, the returned container will also be ordered. This is the union operator.

CONTAINER - CONTAINER
   Removes occurences of right-hand side in left-hand side, and returns the result. For map, keys occuring on the right-hand side will be removed from the left hand side.

CONTAINER disjoint CONTAINER
   Returns true if the two sets have no element in common. For map this means no common key.

CONTAINER intersect CONTAINER
   Returns elements common to both CONTAINERS. For map, this returns key value pairs where key occurs in both maps.

SET repeat UNSIGNED
   Repeat CONTAINER UNSIGNED number of times.

first CONTAINER
   Returns first element of CONTAINER. This requires the container to be ordered (sortable).

last CONTAINER
   Returns last element of CONTAINER. This requires the container to be ordered (sortable).

CONTAINER every UNSIGNED
   Returns every UNSIGNED element of CONTAINER. This requires the CONTAINER to be ordered (sortable).

LIST at UNSIGNED
   Returns element UNSIGNED in LIST, first item is 0.

MAP at ANY
   Returns value in MAP associated with ANY.

CONTAINER apply OPERATOR
   Goes through each item in CONTAINER and puts operator between the elements, and returns the result.
   If CONTAINER is ordered, this will be done in the order given by CONTAINER. If CONTAINER is unordered, then OPERATOR must be commutative, i.e. it must not matter in which order the items are processed.

ANY = ANY
   Returns true if the two objects has the same value, false otherwise.

type ANY = type ANY
   Returns true if the two objects has the same types, false otherwise.

SORTABLE < SORTABLE
   Returns true if the left-hand object sorts less than the right-hand object, false otherwise.

SORTABLE > SORTABLE
   Returns true if the left-hand object sorts greather than the right-hand object, false otherwise.

not BOOLEAN
   Returns true if BOOLEAN is false, false otherwise.

BOOLEAN and BOOLEAN
   Returns true if both BOOLEAN are true.

BOOLEAN or BOOLEAN
   Returns true if one or both of BOOLEAN is true.

BOOLEAN xor BOOLEAN
   Returns true if one and only one of BOOLEAN is true.

SORTABLE <=> SORTABLE
   Returns less if left-hand side sorts less than right-hand side, equal if objects sorts equal, or greater if left-hand side sorts greater than right-hand side.

Immediate sets
--------------

Specify a set using "," operator. A single item is equivalent to a set containing same single item.

Ranges
------

ANY .. ANY
   Both objects must be sortable. Creates a set of object starting with the first object until, but not including, the second.

nil .. ANY
   Creates a list of objects of type ANY starting with lowest possible until, but not including ANY.

ANY .. nil
   Creates a list of objects of type ANY starting with ANY until highest posible.

Error handling
--------------

There are two ways to handle errors in Sisdel: Return error object or throw exception.

Error object
~~~~~~~~~~~~

The error object is special in that all operators are expected to be able to return it unless stated otherwise, and no operator is expected to be able to have it as input unless stated otherwise. Receiving an error object does not cause an error at the caller end, but trying to supply an operator with an error object that cannot handle it will.

When an error is caused by attempting to use an error object when the operator cannot handle it, then the current scope is exited with the error object as evaluated value, i.e. the error object is propagated. This repeats until there is no more scope to exit in which case a default handler is invoked that handles it, typically by logging it and/or printing it.

Exception
~~~~~~~~~

When throwing an exception it is a request for help. The code has ended up in a corner where it does not know how to get out of. Typical example would be out of memory. An exception object is thrown, and the closest defined exception handler receives it. The handler can choose between handling the exception, which means that the error has been sorted out, e.g. more memory allocated, so the execution can continue, or the handler can skip the exception and hope that the next higher exception handler can handle it.

This means that when a code throws an exception, either the program will continue since the issue has been sorted out, or the program will terminate because no handler could handle the exception.

Syntax Playground
=================

## Switch expression

Due to the base syntax of the language, a special switch statement is not needed. Instead, switch can be written in the following way:

    myvar
    	= int then print ( '(is int)' nl )
    	= match ( int ( [ space | tab ] * '(,)' [ space | tab ] int ) * ) then print '(is list of int)'
    	< 0 then print ( '(is negative)' nl )

If myvar is a negative integer, the above will print "is int" as well as "is negative". Since a block of statements is by default a set of statements, there is no priority between them. This means that all statements are evaluated, and must not be a dependency on the order. The expressions are however executed in the order given, i.e. "is int" will be printed before "is negative" for a negative integer.

If `true then <expression>` is used then this will always be run. If this expression is placed last in the block of statements, it will be executed after any other match.

If you want the statements evaluated in the order given, make the block of statement a list by simply adding the `list` operator:

    myvar list
    	= int then print ( '(is int)' nl )
    	= match ( int ( [ space | tab ] * '(,)' [ space | tab ] int ) * ) then print '(is list of int)'
    	< 0 then print ( '(is negative)' nl )

If `true then <expression>` is used then this will be a catch all, i.e. if no other expression matched this expression will be executed.

## then operator

Syntax:

    <boolean> then <expression>

<expression> is executed when <boolean> evaluates to true. The expression returns the result of <expression> if executed, or `nil` otherwise.

Example:

    a > b then print ( '(a is greated than b)' nl )

## /? Match expression optional

/([)/ /?

Means zero or one [ character

## ! Assertion operator

a < b !

## : Assignment operator

a : 5

## <=> compare operator

a <=> b ?
	> print "(larger)"
	= print "(equal)"
	< print "(smaller)"

Rules
=====

1. Types can be fully specified, partly specified or not specified at all
2. Operators are context sensitive, i.e. what operator that will be invoked depends on type for current context
3. If type of current context allows several operator implementations, this is a compile error

References
==========

- Elements of Programming
  http://elementsofprogramming.com/eop_bluelinks.pdf

