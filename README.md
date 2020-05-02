# Beancount
[![Build Status](https://travis-ci.org/twilco/beancount.svg?branch=master)](https://travis-ci.org/twilco/beancount)
[![Join the chat at https://gitter.im/beancount-rs/community](https://badges.gitter.im/Join%20Chat.svg)](https://gitter.im/beancount-rs/community?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

Rust tooling surrounding [beancount](https://github.com/beancount/beancount), a text-based double-entry bookkeeping system.

This repository will provide three main things:

1. A crate (`beancount-core`) that contain core data structures for representing beancount data.
2. A crate (`beancount-parser`) that will parse valid beancount input and output it's representation as Rust data structures.
3. A compile-time type-checked builder API for all Beancount constructs that can be converted to valid Beancount output (file, string, whatever).
4. A crate (`beancount-sys`) that provides bindings to the [Python beancount library](https://github.com/beancount/beancount), likely through the use of [pyo3](https://github.com/PyO3/pyo3).  If deemed useful, a crate that exposes higher-level beancount bindings will be created.

This project is very much in its early stages.  If any of these things interest you feel free to contact me and/or submit a PR!

## License

This work is licensed under Apache/2 or MIT license, per your choice. All contributions
are also given under the same license.
