# rusty-bootstrapper

An alternative, Rust-based bootstrapper for the SKCraft launcher.

Currently in development. It does work but I don't recommend you
distribute this to your users.

## why?

The problem with the Java bootstrapper is that it can silently fail
when the user has an incorrect Java version. Most end-users don't know
what version of Java they have; even Minecraft players may not know
because Mojang's modern launcher bundles Java.

For this reason both the Java bootstrapper and launcher target Java 6,
in order to have the highest chance of working. But the projects can
also break on too *new* Java versions, when not updated.

This project tries to fix that in a couple ways:

- Written in Rust, has no hard dependency on the Java runtime
- Uses native platform UI kits wherever possible
    - winapi on Windows, Cocoa on Mac
    - `zenity`/`kdialog` command line tools on Linux
- Reports errors via dialog wherever possible to avoid silent failure
- Reports when the launcher exits with an error
- Can verify Java version and report incompatibilities (TODO, some
implementation is done)

## building

no compile-time config file just yet so this project isn't super
usable. soon!

### I want to build it anyway

ok it requires the rust toolchain, probably nightly

and take a look at the TODOs in `src/main.rs` for config

```sh
cargo build --release
```
