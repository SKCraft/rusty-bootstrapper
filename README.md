# rusty-bootstrapper

An alternative, Rust-based bootstrapper for the SKCraft launcher.

Currently in development. It does work but I don't recommend you
distribute this to your users.

**Note: This bootstrapper is not a direct drop-in replacement for
the original bootstrapper.** It works differently & it uses different
data directory locations.

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

The default config file lives at `src/settings.json`. You need to edit
this file:

- `update_url` should be a HTTP URL that returns an update JSON.
- `org_name` should be the name of your organization, e.g. "My Company"
- `app_name` should be the name of the app, e.g. "My Company Launcher"

| Platform | Launcher Data Location |
| -------- | ----------------- |
| Windows  | `%APPDATA%/My Company/My Company Launcher/data` |
| Mac      | `$HOME/Library/Application Support/My-Company.My-Company-Launcher` |
| Linux    | `$XDG_DATA_HOME/mycompanylauncher` |

The update JSON returned from `update_url` should look like:

```json
{
  "url": "http://your.website/launcher-4.0.0.jar",
  "version": "4.0.0"
}
```

Once you've edited the configuration simply run:

```sh
cargo build --release
python3 append_data.py target/release/rusty-bootstrapper
```
On Windows, you must add `.exe` to the end of the python script:
```sh
python3 append_data.py target/release/rusty-bootstrapper.exe
```

(The `append_data.py` step is simple helper script that appends
your settings to the end of the binary, which is picked up later by
the bootstrapper. This will be a helpful tool rather than a Python
script in the future.)

The executable will be in `target/release/`.

Note that these instructions only build an executable for your
current platform. Methods for building for multiple platforms
will be available on release.
