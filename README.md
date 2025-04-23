# Rusting Bitcoin Seminar by Vinteum

Welcome to the Rusting Bitcoin Seminar by Vinteum.
This is an 8-week seminar designed to help you develop skills in the Rust programming language within the context of the Bitcoin ecosystem.

## Hands-on methodology

Around 88% of programmers prefer to learn a new language or concept by building something with it[^1].
So, the primary component of the Rusting Bitcoin Seminar is a set of eight programming activities.
These are designed to guide you through implementing a non-trivial piece of software from scratch using many of Rust's core features.

Step by step, you’ll build your own version of the [Bitcoin Seeder](https://github.com/sipa/bitcoin-seeder)—a tool that helps new Bitcoin nodes discover active peers and bootstrap into the network by providing IP addresses of reachable nodes.
This project strikes a balance:
it’s simple enough to complete within a few weeks, yet complex enough to challenge and expand your skills.

## Systems programming focus

Most developers work in *application development*[^2], building software that delivers services directly to end users.
As a result, the languages they typically use (like JavaScript, Java, or C#) prioritize portability and safety over low-level control.

By contrast, *systems programming* involves writing software that provides services to other software, operates under performance constraints, or both—think operating systems, databases, networking stacks, or embedded firmware.
Rust is a modern systems programming language.
It emphasizes control over hardware and performance without sacrificing safety and expressiveness, thanks to its powerful type system.

Of course, we need robust applications built on top of Bitcoin.
But when we talk about *Bitcoin development*, we’re often referring to implementing *communication protocols*—a classic systems programming challenge.

Rust bridges both worlds:
while it supports writing full-featured applications, it was specifically designed to expose and control features often abstracted away in application-focused languages—especially *memory management*, *data representation*, and *concurrency*.

Here’s what makes Rust stand out:

- Rust’s type system is based on the [Hindley–Milner type system](https://en.wikipedia.org/wiki/Hindley%E2%80%93Milner_type_system), which corresponds to a logical foundation[^3].
In practice, this means you can use types to describe your problem domain and rely on the compiler to verify correctness.
  
- It features *traits* and trait implementations, extending polymorphism in a way inspired by Haskell’s typeclasses—supporting advanced features like associated types.

- Rust’s type system is *linear*, meaning values must be “used once.” The **borrow checker** ensures you can have many immutable references or a single mutable one—but never both at the same time.
This enables Rust to produce efficient, memory-safe code and gives you a first-class notion of mutability.
It can be tricky to get used to, but it’s an extremely liberating discipline.

## Our Learning Philosophy

This seminar takes a flipped-classroom approach:
the lectures are not the main event.
Instead, they serve as optional touchpoints to reinforce ideas, clarify concepts, and open space for discussion.
The real learning happens through doing—by tackling each week's programming challenge and exploring the references on your own.
Think of the seminar as guided self-study, with just enough structure and support to keep you moving forward.

To support this methodology, the seminar also includes a series of eight lectures.
These sessions are opportunities to delve deeper into both theoretical and practical aspects of Rust and systems programming, helping you connect ideas, ask questions, and share your progress with others.

## References

### Books on Rust

1. [The Rust Book](https://doc.rust-lang.org/stable/book/)
   The official Rust learning resource.
2. [Jim Blandy et al; *Programming Rust*, 3rd Edition](https://www.oreilly.com/library/view/programming-rust-3rd/9781098176228/)
   A practical and comprehensive guide to Rust for systems programming.
3. [Ken Youens-Clark; *Command-Line Rust*](https://www.oreilly.com/library/view/command-line-rust/9781098109424/)
   Learn Rust by recreating classical Unix command-line tools like `grep`, `ls`, and `tail`.

### Books on Bitcoin

1. [Andreas Antonopoulos and David Harding; *Mastering Bitcoin*](https://github.com/bitcoinbook/bitcoinbook?tab=readme-ov-file)
   The unofficial go-to book for understanding Bitcoin.
2. [Jimmy Song; *Programming Bitcoin*](https://www.oreilly.com/library/view/programming-bitcoin/9781492031482/)
   Learn the Bitcoin protocol by implementing it yourself. A bit dated, but still great for learning the fundamentals.

### Other resources

1. [Awesome Rust](https://github.com/rust-unofficial/awesome-rust)
2. [Awesome Bitcoin](https://github.com/igorbarinov/awesome-bitcoin)

---

[^1]: "I only believe in statistics that I doctored myself." — Winston Churchill
[^2]: "There are two kinds of statistics, the kind you look up and the kind you make up." — Rex Stout
[^3]: [Philip Wadler, *Propositions as Types*](https://homepages.inf.ed.ac.uk/wadler/papers/propositions-as-types/propositions-as-types.pdf)
