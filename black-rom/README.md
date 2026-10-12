# Pokémon Black

[![Progress]][decomp.dev] [![Report]][actions]

Discord: https://discord.gg/x9YSZECNVV

MelonDS: https://melonds.kuribo64.net/downloads.php

This is a decompilation of Pokémon Black (NDS).

It builds the following ROM:

**pokeblack.us.nds** `sha1: 26ad0b9967aa279c4a266ee69f52b9b2332399a5` (Version 1.0 USA/EUR NDSi Enhanced)

[Progress]: https://decomp.dev/squiddonaut/pokeblack.svg?mode=shield&label=Code
[decomp.dev]: https://decomp.dev/squiddonaut/pokeblack
[Report]: https://github.com/squiddonaut/pokeblack/actions/workflows/report.yml/badge.svg
[actions]: https://github.com/squiddonaut/pokeblack/actions/workflows/report.yml

## MacOS Setup & Build

The CodeWarrior compiler is a 32-bit Windows binary and Homebrew no longer ships a usable Wine, so on macOS the build runs inside an x86 Linux Docker container (`Dockerfile.wine`).

1. Install Docker Desktop and the ARM binutils, then start Docker:

   ```bash
   brew install --cask docker
   brew install arm-none-eabi-binutils
   open -a Docker
   ```

2. Download these files from the [v0.0.0 release](https://github.com/spencerlepine/black/releases/tag/v0.0.0) (originals are in `#resources` in the Discord, see [INSTALL.md](INSTALL.md)):
   - [`mwccarm.zip`](https://github.com/spencerlepine/black/releases/download/v0.0.0/mwccarm.zip)
   - [`TwlSDK-5_3-1-20090824.zip`](https://github.com/spencerlepine/black/releases/download/v0.0.0/TwlSDK-5_3-1-20090824.zip)
   - [`libsyscall.zip`](https://github.com/spencerlepine/black/releases/download/v0.0.0/libsyscall.zip) (not needed for the build)
   - [`pokeblack.us.nds`](https://github.com/spencerlepine/black/releases/download/v0.0.0/pokeblack.us.nds) (prebuilt ROM, to skip building and just play)

   Extract the zips from the repo root:

   ```bash
   unzip -q ~/Downloads/mwccarm.zip 'mwccarm/dsi/1.1/*' 'mwccarm/license.dat' -d /tmp/mwccarm
   cp -R /tmp/mwccarm/mwccarm/. tools/mwccarm/

   unzip -q ~/Downloads/TwlSDK-5_3-1-20090824.zip -d /tmp/twlsdk
   mkdir -p lib/NitroSDK
   mv /tmp/twlsdk/TwlSDK-5_3-1-20090824/TwlSDK lib/NitroSDK/TwlSDK
   rm -rf /tmp/mwccarm /tmp/twlsdk
   ```

3. Build the image (once) and the ROM:

   ```bash
   docker build -f Dockerfile.wine -t pokeblack-wine .
   docker run --rm -v "$PWD":/pokeblack pokeblack-wine make tools   # once
   docker run --rm -v "$PWD":/pokeblack pokeblack-wine make check-toolchain
   docker run --rm -v "$PWD":/pokeblack pokeblack-wine make -j4
   ```

   The build ends with `ROM matches black.us/rom.sha1`. Expect harmless Wine `winediag` noise and some linker warnings. Use `make compare` instead of `make` for a fully hash-checked build.

4. Play `build/black.us/pokeblack.us.nds` in an emulator:

   ```bash
   brew install --cask melonds
   ```
