# guzhidui

_A simple static file server_

`guzhidui` is a very simple static file server,
written in Rust, built on top of the [Iron](https://github.com/iron/iron) framework,
and the iron-archivist library.
best suited for the serving of repositories
that uses file system directory structure to organise its contents.

This project is still in alpha.
Please proceed with caution.

Run
```
cargo run -- --config guzhidui.toml
```
to set up a server that serves the contents of the current directory,
using configurations specified in `guzhidui.toml`.

See `guzhidui.toml` for all configuration options.
Run with the `--help` argument for an overview of available command line arguments.

