Dependent Type Theory
---------------------
Key Idea
   Dependent types allow types to depend on values. This means that types can be parameterized by values, enabling precise specifications of data structures, functions, and properties.

Example
   In a dependent type system, you could have a type for lists where the length of the list is encoded in the type itself. For example, a type might represent "list of length n," where n is a natural number.

Applications
   Dependent types are particularly useful for specifying and verifying properties of programs, such as correct array indexing, dimensionality of matrices, or even correctness properties of algorithms.

Homotopy Type Theory (HoTT)
---------------------------
Key Idea
   Homotopy type theory connects type theory with homotopy theory from topology. It introduces the concept of higher-dimensional types, where paths between objects can have non-trivial topological structures.

Example
   In HoTT, types are interpreted as spaces, and elements of types correspond to points in those spaces. Paths between points can be identified in higher-dimensional spaces, providing a rich structure for reasoning about equivalences and transformations.

Applications
   HoTT has applications in both mathematics and computer science, offering new perspectives on logic, category theory, and formalization of mathematics.

Modal Type Theory
-----------------

Key Idea
   Modal type theory extends traditional type theory with modal operators inspired by modal logic. These operators allow reasoning about necessity, possibility, or other modalities within the type system.

Example
   Modal operators such as ◇ (diamond) for possibility and ◻ (box) for necessity can be used to express properties that hold under different modalities. For example, a type might represent "possible integers" or "necessary proofs."

Applications
   Modal type theory is useful for expressing and reasoning about various kinds of modal properties in programming languages, formal verification, and reasoning about knowledge and belief in epistemic logic.

Categorical Type Theory
-----------------------
Key Idea
   Categorical type theory connects type theory with category theory, a branch of mathematics concerned with abstract structures and relationships between them. It provides a categorical foundation for types and functions.

Example
   In categorical type theory, types are interpreted as objects in categories, and functions between types are interpreted as morphisms in those categories. This provides a unified framework for understanding types and functions across different mathematical structures.

Applications
   Categorical type theory has applications in formalizing mathematical structures, programming language semantics, and reasoning about abstract properties of programs and systems.

Linear Type Theory
------------------
Key Idea
   Linear type theory imposes restrictions on resource usage by allowing each variable to be used exactly once. This ensures that resources, such as memory or communication channels, are used in a controlled manner.

Example
   In linear type theory, a function that consumes a resource must explicitly consume it, ensuring that the resource is not inadvertently used again elsewhere in the program.

Applications
   Linear type theory is useful for reasoning about resource management, memory safety, protocol adherence, and other properties related to resource consumption in programming languages and systems.

Intersection Type Theory
------------------------
Key Idea
   Intersection type theory extends traditional type theory by allowing types to be combined via intersection, meaning an expression may belong to multiple types simultaneously.

Example
   An intersection type might represent the combination of two separate types, such as "integer" and "positive integer," ensuring that an expression satisfies both properties simultaneously.

Applications
   Intersection type theory is useful for expressing and reasoning about complex type constraints, refinement types, and precise specifications of program behavior.

Intentional Type Theory
-----------------------
Key Idea
   Intentional Type Theory focuses on the intentional aspects of types, meaning it emphasizes how types are intended to represent specific computational or logical concepts rather than just their extensional properties (i.e., what values they contain). It aims to capture the intended meaning and behavior of programs or proofs through their types, providing a precise and expressive framework for reasoning about programs and formal systems.

Example
   In Intentional Type Theory, types are not merely collections of values but are imbued with intentional properties that reflect their intended use and semantics. For example, consider a type representing sorted lists of integers. The type definition not only specifies the structure of the data (a list) but also captures the intended property that the elements are arranged in ascending order. This intentional aspect of the type guides how programs manipulate and reason about values of that type.

Applications

   1. **Program Verification**: Intentional Type Theory provides a foundation for formal methods and program verification techniques by allowing types to express not only data structures but also invariants and properties of programs. This facilitates rigorous reasoning about program correctness and behavior.
   2. **Language Design and Implementation**: Intentional Type Theory influences the design and implementation of programming languages by providing insights into how types can capture the intended semantics of language constructs. It helps language designers ensure that type systems align with the intended usage and semantics of the language features.
   3. **Proof Theory and Formal Logic**: In the context of formal logic, Intentional Type Theory aids in the development of proof systems by providing a framework for specifying the intentional content of logical propositions and inference rules. This contributes to the development of reliable and expressive formal reasoning systems.

Parametric Type Theory
----------------------
Key Idea
   Parametric type theory extends traditional type theory by introducing parametric polymorphism. The key idea is to allow functions and data structures to be defined in terms of type parameters, enabling generic programming and abstraction over types. Instead of specifying a fixed type for a function or data structure, parametric type theory allows them to be polymorphic, meaning they can operate uniformly over different types.

Example
   Consider a simple example of a parametrically polymorphic function in a parametric type theory. Let's say we want to define a function `identity` that takes an argument of any type and returns the same argument:

   .. code-block:: haskell

      identity :: forall a. a -> a
      identity x = x

   Here, `identity` is parametrically polymorphic over type `a`. It can accept arguments of any type `a` and returns the same value without performing any operations specific to the type. The `forall a` quantifier indicates that `identity` is universally quantified over all types `a`.

Applications
   1. **Code Reusability**: Parametric type theory promotes code reuse by enabling the definition of generic functions and data structures that operate uniformly over different types. This reduces code duplication and improves maintainability by allowing developers to write generic algorithms that work for a wide range of data types.
   2. **Abstraction**: Parametric polymorphism allows programmers to write more abstract and general code by decoupling algorithms from specific types. This encourages higher levels of abstraction and modularity in software design, leading to more flexible and reusable codebases.
   3. **Type Safety**: Parametric type theory enhances type safety by providing stronger guarantees about the behavior of polymorphic functions and data structures. Since parametrically polymorphic code works uniformly over all types, type errors are caught at compile-time, reducing the likelihood of runtime errors and improving program reliability.
   4. **Performance**: Parametric polymorphism can also lead to performance benefits by enabling compiler optimizations such as monomorphization, where specialized versions of polymorphic functions are generated for specific types at compile-time. This eliminates the overhead associated with dynamic dispatch and can result in more efficient code execution.

In summary, parametric type theory facilitates generic programming and abstraction by introducing parametric polymorphism, which allows functions and data structures to be defined in terms of type parameters. This promotes code reusability, abstraction, type safety, and potential performance improvements in software development.

Session Type Theory
-------------------
Key Idea
   Session type theory provides a formal framework for specifying communication protocols and interactions between concurrent processes. It extends traditional type theory with constructs for describing communication patterns, ensuring that concurrent programs adhere to specified protocols and exhibit desired communication behaviors.

Example
   Consider a simple example of a session type specifying a client-server interaction protocol:

   .. code-block:: session

      protocol ClientServer {
          request : Int -> String;
          response : String -> Int;
      }

   Here, `ClientServer` represents a communication protocol between a client and a server. The `request` and `response` labels indicate the types of messages that can be exchanged between the client and server processes. For example, the client sends an integer to the server using the `request` label, and the server responds with a string using the `response` label.

Applications
   1. **Protocol Specification**: Session type theory enables precise specification of communication protocols between concurrent processes, ensuring that interactions adhere to specified patterns and constraints. This helps in designing reliable and maintainable distributed systems by providing a formal foundation for specifying and verifying communication behaviors.
   2. **Protocol Verification**: Session types facilitate formal verification of concurrent programs by statically checking whether the communication patterns conform to specified protocols. This helps in detecting and preventing communication errors, such as deadlocks, message mismatches, or protocol violations, at compile-time.
   3. **Concurrency Control**: Session type theory aids in managing concurrency and coordinating interactions between concurrent processes by enforcing communication protocols. By ensuring that processes follow specified communication patterns, session types promote safe and predictable concurrency, reducing the likelihood of communication-related bugs and errors.
   4. **Distributed Systems**: Session type theory is particularly useful in the design and implementation of distributed systems, where communication between distributed components is crucial. By providing a formal framework for specifying and verifying communication protocols, session types help in building reliable and scalable distributed systems that exhibit predictable communication behaviors.

In summary, session type theory extends traditional type theory with constructs for specifying communication protocols between concurrent processes. It is applied in various areas such as protocol specification, verification, concurrency control, and the design of distributed systems, facilitating the development of reliable and maintainable concurrent software.

Behavioral Type Theory
----------------------
Key Idea
   Behavioral type theory focuses on specifying and verifying the dynamic behavior of programs through types. It extends traditional type theory with constructs for expressing temporal and behavioral properties of programs, enabling rigorous analysis of program behavior and correctness properties.

Example
   Consider a simple example of a behavioral type specifying the behavior of a producer-consumer communication pattern:

   .. code-block:: behavioral

      producerConsumer : {produce : Int -> consume : Int -> End}

   Here, `producerConsumer` represents a behavioral type that describes a communication pattern between a producer and a consumer. The type specifies that the producer can produce an integer and then the consumer can consume the same integer. The `End` symbol indicates the termination of the communication pattern.

Applications
   1. **Behavioral Specification**: Behavioral type theory enables precise specification of program behavior and communication patterns, allowing developers to express temporal and dynamic properties of programs through types. This helps in clarifying program semantics and requirements, facilitating better understanding and communication among developers.
   2. **Program Verification**: Behavioral types aid in the formal verification of programs by statically checking whether program executions conform to specified behavioral properties. By encoding behavioral constraints into types, behavioral type theory enables automated analysis and verification of program behavior, helping in detecting and preventing errors and bugs.
   3. **Concurrency Control**: Behavioral type theory is valuable for managing concurrency and coordinating interactions between concurrent processes by enforcing behavioral constraints. By ensuring that processes adhere to specified behavioral patterns, behavioral types promote safe and predictable concurrency, reducing the likelihood of concurrency-related errors and race conditions.
   4. **Domain-Specific Modeling**: Behavioral type theory is useful in domain-specific modeling and specification languages where the dynamic behavior of systems is crucial. By providing a formal framework for expressing temporal and behavioral properties, behavioral types enable precise modeling and analysis of system behavior, leading to more reliable and maintainable software systems.

In summary, behavioral type theory extends traditional type theory with constructs for specifying and verifying the dynamic behavior of programs through types. It finds applications in behavioral specification, program verification, concurrency control, and domain-specific modeling, facilitating the development of reliable and correct software systems.

Substructural Type Theory
-------------------------
Key Idea
   Substructural type theory relaxes the structural rules of traditional type theory, allowing for more flexible and fine-grained control over resource usage. It includes substructural logics such as linear logic and affine logic, which impose constraints on the use of resources like memory or communication channels.

Example
   Consider a simple example of a substructural type specifying a linearly typed function:

   .. code-block:: substructural

      linearFunction : !Int -> !Int

   Here, `linearFunction` represents a linearly typed function that consumes an integer argument and produces an integer result. The `!` symbol indicates that the integer argument must be used exactly once, enforcing linear usage of resources.

Applications
   1. **Resource Management**: Substructural type theory is valuable for managing resources such as memory, communication channels, or file handles in programming languages. By enforcing constraints on resource usage through types, substructural types promote safe and efficient resource management, reducing the likelihood of resource leaks or misuse.
   2. **Concurrency Control**: Substructural type theory aids in managing concurrency and coordinating interactions between concurrent processes by enforcing resource usage constraints. By specifying how resources are consumed and shared among processes, substructural types help in preventing race conditions, deadlocks, and other concurrency-related errors.
   3. **Security and Safety**: Substructural types contribute to enhancing the security and safety of software systems by enforcing access control policies and resource usage restrictions. By statically verifying resource usage patterns, substructural type theory helps in detecting and preventing security vulnerabilities, such as buffer overflows or unauthorized access to resources.
   4. **Optimizations**: Substructural type theory enables compiler optimizations such as memory management and code reuse by providing information about resource usage patterns. By analyzing substructural types, compilers can perform optimizations such as memory deallocation, resource pooling, or code elimination, leading to more efficient and optimized code execution.

In summary, substructural type theory extends traditional type theory by imposing constraints on resource usage through substructural logics. It finds applications in resource management, concurrency control, security, safety, and compiler optimizations, contributing to the development of reliable, efficient, and secure software systems.

Unified Type Theory
-------------------
Key Idea
   Unified type theory seeks to unify different aspects of type theory, such as dependent types, polymorphism, and subtyping, into a coherent framework. It aims to provide a unified foundation for expressing a wide range of programming language features and properties within a single type-theoretic framework.

Example
   Consider a simple example of a unified type that combines dependent types, polymorphism, and subtyping:

   .. code-block:: unified

      List<A> : Type -> Type

   Here, `List<A>` represents a polymorphic type constructor that generates list types parameterized by the element type `A`. The type `List<A>` is dependent on the value of `A`, allowing for the definition of lists of arbitrary types. Additionally, `List<A>` can be used in a subtype relationship, where lists of different element types are considered subtypes of each other.

Applications
   1. **Expressive Type Systems**: Unified type theory enables the design of expressive type systems that integrate various type-theoretic features such as dependent types, polymorphism, and subtyping. By unifying these features into a single framework, unified type theory provides a rich and flexible language for expressing complex type relationships and properties.
   2. **Language Design and Implementation**: Unified type theory influences the design and implementation of programming languages by providing insights into how different type-theoretic features can be integrated and harmonized. It helps language designers create languages with powerful and expressive type systems that support a wide range of programming paradigms and styles.
   3. **Formal Methods and Verification**: Unified type theory aids in the development of formal methods and verification techniques by providing a unified foundation for expressing properties and specifications within a single type-theoretic framework. By leveraging the expressive power of unified type theory, formal methods researchers can specify and verify complex properties of programs and systems more effectively.
   4. **Program Analysis and Optimization**: Unified type theory facilitates program analysis and optimization by providing a unified representation of program types and structures. By analyzing programs within the unified type-theoretic framework, compilers and analysis tools can infer program properties, perform type-based optimizations, and generate more efficient code.

In summary, unified type theory seeks to unify different aspects of type theory into a coherent framework, providing a unified foundation for expressing programming language features and properties. It finds applications in designing expressive type systems, language design and implementation, formal methods, program analysis, and optimization.

Intrinsic Types
---------------

Start with the type, then check that the code fulfills type requirements. Good choice if types affect how code behaves.

Extrinsic Types
---------------

Looks at the code and determines what types that should correspond to. Good choice if type need to be worked out as you go.

Topal
=====

Use lazy intrinsic types. Types do affect behavior, but sometimes you still don't know upfront what type something should have. Use a set or range of possible types, and as code progress, remove types that would not make sense. When all code has been parsed, the type will be the choice or choices that was left.

Links
=====

Lambda Calculus Cube
    https://youtu.be/UCzE15Hvs1E?si=7YHA9sE0keI-k2eB

