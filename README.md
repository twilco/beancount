# Beancount

[![Join the chat at https://gitter.im/beancount-rs/community](https://badges.gitter.im/Join%20Chat.svg)](https://gitter.im/beancount-rs/community?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

Rust tooling surrounding [Beancount](https://github.com/beancount/beancount), a text-based double-entry bookkeeping system.

This repository contains three crates currently.

1. `beancount-core`, which contains a compile-time type-checked builder API and core data structures for representing Beancount data.
2. `beancount-parser`, which parses valid Beancount input and will output it's representation as Rust data structures.
3. `beancount-render`, which can format the beancount structures and output it via anything that implements `Write`.

This repository will also provide:

1. A crate (`beancount-sys`) that provides bindings to the [Python Beancount library](https://github.com/beancount/beancount), likely through the use of [pyo3](https://github.com/PyO3/pyo3). If deemed useful, a crate that exposes higher-level Beancount bindings will be created. With this work, we could hopefully unlock the ability to integrate with existing Python Beancount plugins.

If any of these things interest you feel free to contact me and/or submit a PR!

## License

This work is licensed under Apache/2 or MIT license, per your choice. All contributions
are also given under the same license.
