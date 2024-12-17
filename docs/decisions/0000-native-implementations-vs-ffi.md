---
status: proposed
date: 2024-12-17
decision-makers: Joe Polny, David Rojas, Bruno Martins
consulted: Algorand Foundation CTO office, MakerX engineering team
informed: N/A
---

# Native Implementations vs FFI for Core Algorand Functionality

## Context and Problem Statement

It is desireable for the Algorand Foundation to create and maintain libraries for multiple languages that provide core Algorand functionality. These libraries should be able to help developers use Algorand but also not reinvent the wheel by trying to handle functionality that is already available in external libraries (ie. keypair generation, signing, etc.). There should be consistency across languages so that the API is relatively similar and there is not one language that gets ahead or falls behind the others in terms of features, as we've seen with the existing SDKs and Algokit Utils libraries. The exact languages we want to target are TBD, but ideally we can get the most popular languages for popular platforms (Python, JS for web and node, Kotlin for Android, Swift for iOS).

## Decision Drivers

- The chosen approach should be easy to implement and maintain
- The chosen approach should enable cross-language consistency
- The chosen approach should enable a sane API for core Algorand functionalities (exact list to be determined later)

## Considered Options

- Native implementations in each language
- Core Rust implementation with hand-written FFI bindings
- Core Rust implementation with generated FFI-bindings using UniFFI and wasm-pack
- Core Rust implementation with generated FFI-bindings using language-specific binding generators

## Decision Outcome

TBD. Currently leaning towards "Core Rust implementation with generated FFI-bindings using UniFFI and wasm-pack" with the intitial spike in [./algo_models_rs](https://github.com/algorand-foundation/algo_models_rs).

### Confirmation

Research will be done on the FFI/WASM toolchains and an initial spike will be done to determine the feasibility of the approach.

The spike of the solution(s) should explore the following:

- Implementation/maintenance burden
- Performance
- Toolchain maturity
- Developer experience
- Cross-platform support
- Packaging

## Pros and Cons of the Options

### Native implementations in each language

- Good, because we can fully leverage the language's capabilities and features
- Neutral, because we can use some existing Algorand libraries for some functionality
- Bad, because it will be a significant implementation and maintenance burden

### Core Rust implementation with hand-written FFI bindings

One rust implementation with one FFI interface that is used by all languages.

- Good, because we have one core implementation to maintain
- Good, because it will be easier to maintain consistency across languages
- Bad, because Rust is new to the Algorand Foundation engineering team
- Bad, because there is a cost to crossing the FFI boundary, paticularly for light compute functions
- Bad, because writing C bindings can get quite difficult with complex types

### Core Rust implementation with generated FFI-bindings using UniFFI and WASM

One rust implementation with one FFI interface that is used by all languages.

- Good, because we have one core implementation to maintain
- Good, because it will be easier to maintain consistency across languages
- Good, because we can leverage the UniFFI toolchain to generate bindings for multiple languages (Python, Kotlin, Swift... potentially more)
- Good, because we can expect the UniFFI toolchain to mature and get more bindings over time (third-party bindings for other languages)
- Good, because if we want to target other languages, we can use the UniFFI framework to write binding generators
- Good, because we can leverage the wasm-pack toolchain to generate bindings for JS (node and browser)
- Neutral, these toolchains are fairly new (both v0)
- Neutral, all langauges have to have the exact same API and type definitions, even if it means not taking advantage of the language's capabilities
- Bad, because Rust is new to the Algorand Foundation engineering team
- Bad, because there is a cost to crossing the FFI boundary, paticularly for light compute functions
- Bad, because we have to accept some of the [performance tradeoffs](https://github.com/mozilla/uniffi-rs/blob/main/docs/adr/0002-serialize-complex-datatypes.md) (made in favor of safety) of UniFFI
- Bad, because we will need to ship binaries with the library for each language

### Core Rust implementation with generated FFI-bindings using language-specific binding generators

One rust implementation with one FFI interface that is used by all languages.

- Good, because we have one core implementation to maintain
- Good, because it will be easier to maintain consistency across languages
- Good, because we can leverage the wasm-pack toolchain to generate bindings for JS (node and browser)
- Good, because we can leverage language-specific binding generators (ie. PyO3 for Python) to generate performant bindings that take advantage of the language's capabilities
- Bad, because Rust is new to the Algorand Foundation engineering team
- Bad, because there is a cost to crossing the FFI boundary, paticularly for light compute functions
- Bad, because there will be a cost to maintaining the interface and bindings for each language
- Bad, because it might be harder to maintain consistency across languages
- Bad, because we will need to ship binaries with the library for each language

## More Information

It should be noted that there are other toolchains similar to UniFFI for generating multi-language FFI bindings. UniFFI seemed like the clear winner based on [Mozilla's ADR](https://github.com/mozilla/uniffi-rs/blob/main/docs/adr/0000-whats-the-big-idea.md) that lead to the creation of UniFFI.
