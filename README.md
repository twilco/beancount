## Beancount

Rust tooling surrounding [beancount](https://github.com/beancount/beancount), a double-entry bookkeeping language.

This repository will provide three main things:

1. A crate (`beancount-parse`) that will parse valid beancount input and output it's representation as Rust data structures.
2. A compile-time type-checked builder API for all Beancount constructs that can be converted to valid Beancount output (file, string, whatever).
3. A crate (`beancount-sys`) that provides bindings to the [Python beancount library](https://github.com/beancount/beancount), likely through the use of [pyo3](https://github.com/PyO3/pyo3).  If deemed useful, a crate that exposes higher-level beancount bindings will be created.

This project is very much in its early stages.  If any of these things interest you feel free to contact me and/or submit a PR!