<div align=center>
    <h1>BEAMinject by wavEye</h1>Memory injection unlocker for Minecraft: Windows 10 Edition</p>
</div>

-----

## :zap: Features
- Written in memory-safe Rust, using [`libdopamine`](https://github.com/wavEye-Project/libdopamine)
- Supports x86, x64, ARM64 and even ARM
- Silent mode for no logging while injecting
- Supports Minecraft Preview releases
- Patching is done in memory, not modifying any system files
- Doesn't affect any other apps, safely injects to Minecraft

## :inbox_tray: Downloads
You can download the latest nightly release [here](https://nightly.link/wavEye-Project/BEAMinject/workflows/build/main/BEAMinject.zip).

## :wrench: Usage
Running BEAMinject as-is will patch Minecraft's release with logging by default.

It has the following features:
- Silent mode
- Minecraft Preview support
- and more to come...

Pass the **`-h`/`--help`** flag to BEAMinject in a command line for more info.

## :warning: Common issues
**"It says it patched it, but it still doesn't work!" >** Give Minecraft ~10s to notice the patched library. Entering and quitting a world works too.

If you have other issues, please create an issue.

## :test_tube: ARM support
The patcher supports ARM from the source. The emulation layer does not affect patching, and ARM versions of the game are still supported.

## :page_with_curl: License
All code and assets are licensed under The Unlicense.
