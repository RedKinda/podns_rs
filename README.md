# Pronouns over DNS - Rust Implementation

This library aims to implement the podns RFC for pronoun resolution over DNS using Rust.
The RFC can be found [here](https://github.com/CutieZone/pronouns-over-dns).

## How to Use

Simplest way to use this is to install it via cargo:

```sh
cargo install --features dns_resolve podns
```

and then run it with a domain name as an argument:

```sh
podns kinda.red
```

This will output the pronouns associated with the given domain name, if available.

To use this library in your own Rust project, add the following to your `Cargo.toml`:

```toml
[dependencies]
podns = { version = "0.1.2", features = ["dns_resolve"] }
```

This project includes a simple DNS resolver using the `resolve` crate, but you are encouraged to bring your own.
You simply need to query for the TXT records of the `pronouns` subdomain, and parse them with `podns::parse_record`.