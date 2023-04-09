====================
Background to Topal
====================

Type-Oriented ProgrAmming Language, or Topal, is intended to be a base for creating domain specific languages as well as being a general purpose language. Its focus is on interoperability and safety.

Why a new language?
===================

After over four decades of programming experience, there are types of problems that tend to reappear again and again, that never seems to get a proper solution. For example how to build your code. Cross-compiling, keep track of dependencies, debugging external code, and more seem like problems that never gets a proper solution.

Many of those problems tend to boil down into problems in the type domain, but on a higher level than current languages typically use. Imagine having types associated with binaries, libraries and data files and what could be solved with that. What if the type system is rich enough to be able to describe how the data in a file should be organized, similar to what CSS files does for HTML. Do the same for describing messages in a message passing system, or hardware registers for a device driver, and things start to get interesting.

Another problem that seem very hard to solve is safety and security. Could a rich type system be able to solve those as well? This language is an attempt to answer that question. Those usually are about resource handling and race conditions. Would it be possible to combine Rust language ability to keep track of who owns what resource and how it can safely be used in a multi-processor environment while not being forced to always specify how to share the resource but let the compiler infer what needs to be done and what the consequences of it are?

Lessons learned from other languages
====================================

As programming languages has evolved so has programming paradigms. Topal takes inspiration from the following paradigms:

- Structued programming
- Object-oriented programming
- Functional programming

Each paradigms removes a degree of freedom from the programmer, since it was deemed unsafe.

Structued programming
   Removed the freedom of explicit transfer of execution context, typcially using goto statement. Transfer of execution context is instead done with structures, e.g. loop structures, conditionals etc.

Object-oriented programming
   Removed the freedom of implicit transfer of execution context using function pointers. Those are instead done using polymorphism or via interface.
