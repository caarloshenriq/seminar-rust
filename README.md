# Rusting Bitcoin Seminar by Vinteum

Welcome to the Rusting Bitcoin Seminar by Vinteum.
This is a 8-week seminar designed to help you develop skills in the Rust programming language in a Bitcoin context.

## Hands-on methodology

Around 88% of programmers prefer to learn a new language or concept by implementing something with it[^1].
So, the first and foremost component of the Rusting Bitcoin Seminar is a set of 8 programming activities designed so that you implement a non-trivial piece of software from scratch using the many features of the Rust language.

Step by step, you'll implement your version of the [Bitcoin Seeder](https://github.com/sipa/bitcoin-seeder), a piece of software that helps Bitcoin Nodes to bootstrap into the Bitcoin network by providing addresses of other active nodes.
This is a sufficiently simple piece of software to be implemented in a few weeks, while being sufficiently complex to push you forward.

## Systems programming focus

Most programmers work in application development[^2], producing and maintaining software which provides services to the users directly.
As a consequence, the programming languages they employ (Javascript, Java, C#, etc) tend to focus on generic features to allow programs written in the language to use the same code on different platforms, trading fine control over the computer resources for generality, convenience and safety.

In constrast, *systems programming* aims to produce software which provide services to other software, are performance constrained, or both (e.g. operating systems, database management systems, networking systems, embedded systems).
Rust is designed as a systems programming language focusing not in compatibility (one code to rule all architectures), but in performance and ease of access to the underlying hardware while still providing high-level programming concepts.

Off course we want and need plenty of applications to be built on top of Bitcoin.
But when we talk about Bitcoin development we are often refering to the implementation of *communication protocols*, a typical systems programming task.

This division between application and systems programming langauges have blurred over time and you surely can write applications in Rust, but the language is designed to exposed and provide control mechanisms over aspects often abstracted away in languages focused on application development, primarily *memory management*, *data representation*, and *concurrency*.
Rust does so by means of a powerful *type system* that provides static (compile time) guarantees.

- Rust's type system is based on the [Hindley–Milner type system](https://en.wikipedia.org/wiki/Hindley–Milner_type_system) which corresponds to a certain logic[^3].
In practice, we can use types to describe the domain of the problem we are trying to solve and use the compiler to check our work.

- Rust has a system of *traits* and instances that extend its polymorphism capabilities that’s almost directly copied from Haskell typeclasses, which even supports fancy features like associated types.

- Rust's type system is linear, which enforces that values are "used once."
The *borrow checker** guarantee you can have any number of immutable references to an object, or one mutable reference, but never both.
This gives Rust the ability to generate very efficient and memory-safe code and also equips Rust with a first-class notion of mutability.
This can be a pain to get used to, but is an extremely liberating discipline.

To account for these unique features, the second component of the Seminar is a series of 8 lectures that discuss theoretical and practical aspects of Rust.

## References

**Books on Rust**

1. [The Rust Book](https://doc.rust-lang.org/stable/book/):
the official Rust learning resource.
2. [Jim Blandy et al; Programming Rust, 3rd Edition](https://www.oreilly.com/library/view/programming-rust-3rd/9781098176228/):
practical and comprehensive book covering all aspects of Rust for systems programming.
3. [Ken Youens-Clark; Command-Line Rust](https://www.oreilly.com/library/view/command-line-rust/9781098109424/):
   learn Rust by recreating classical unix command line tools like `grep`, `ls`,
   and `tail**.

**Books on Bitcoin**

1. [A. Antonopoulos and D. Harding; Mastering Bitcoin](https://github.com/bitcoinbook/bitcoinbook?tab=readme-ov-file):
the unofficial Bitcoin book.
2. [Jimmy Song; Programming Bitcoin](https://www.oreilly.com/library/view/programming-bitcoin/9781492031482/):
learn the Bitcoin protocol by implementing it yourself (a little outdated, but still excellent for learning the basics of the protocol).

**Other resources**

1. [Awesome Rust](https://github.com/rust-unofficial/awesome-rust)
2. [Awesome Bitcoin](https://github.com/igorbarinov/awesome-bitcoin)


[^1]: "I only believe in statistics that I doctored myself." &mdash; Winston Churchill

[^2]: "There are two kinds of statistics, the kind you look up and the kind you make up." &mdash; Rex Stout

[^3]: [Philip Wadler, Propositions as Types](https://homepages.inf.ed.ac.uk/wadler/papers/propositions-as-types/propositions-as-types.pdf)
