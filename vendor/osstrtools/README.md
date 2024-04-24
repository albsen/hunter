osstrtools
======

`OsStrTools` adds some useful methods to `OsStr` and `OsString` that
are missing in the standard library, like
[`split()`](https://docs.rs/osstrtools/trait.OsStrTools.html#tymethod.split),
[`replace()`](https://docs.rs/osstrtools/trait.OsStrTools.html#tymethod.replace),
or
[`splice()`](https://docs.rs/osstrtools/trait.OsStrTools.html#tymethod.splice). It
is mostly useful for dealing dealing with things like file names,
command line arguments, `PathBuf`, and the like.

Windows support is experimental, but shoud hopefully mostly work,
although it is not well tested and likely somewhat slower due to some
overhead since it requires checking the strings for correctness. The
checking is done by
[`os_str_bytes`](https://crates.io/crates/os_str_bytes).

Right now Windows support is still gated behind a feature flag. To
enable it add this to your `Cargo.toml`:



```toml
[dependencies]
osstrtools = { version = "0.2", features = ["windows"] }
```
