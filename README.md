<p align="center">
  <img src="assets/banner.png" alt="Moma banner" width="100%" />
</p>

**Moma** is a Linux-first, CLI **mo**d **ma**nager.

Moma automates everything you need to start modding your games on Linux. It takes care of downloading mods, setting up tools, and managing your configuration in a simple, declarative format, so your mod setup stays clean, trackable, and easy to reproduce.

To keep your game installation untouched, Moma builds a temporary environment where mods are layered only when you launch the game. Your base files remain unchanged, and mods don’t need to be installed directly into the game folder.

Moma avoids Windows-native mod management tools and fully embraces native Linux features. It's designed for minimal, terminal-driven modding without sacrificing control or flexibility.

## Installation

Make sure you have [Rust](https://www.rust-lang.org/tools/install) installed first. Then clone the repo:

```sh
git clone https://github.com/rahmerh/moma.git
cd moma
```

If you want the latest, stable binary check out main first:

```sh
git checkout main
```

If you want latest (and potentially broken), stay on develop.

Run the following to install the binary system-wide:
```sh
sudo make install
```

This will copy the binary to `/usr/local/bin/moma`. You can now run moma from anywhere.

> 💡 **Hint:** You can also install it locally without sudo:
>
> ```sh
> mkdir -p ~/.local/bin
> cp target/release/moma ~/.local/bin
> ```
> Make sure `~/.local/bin` is in your `PATH`.

## Support

Moma only supports certain game platforms and games (and game versions). This is mostly due to that I'm not going to buy games I won't play, so I can't reliably add support for it. If you want a game that isn't on the list, feel free to add an issue to request support. I'd be happy to help.

### Supported game platforms

- Steam (Proton or native)

### Supported games

- Skyrim SE/AE (SKSE modded)

## Usage

### Initialize
```sh
moma init
```

Starts the game setup wizard, guiding you through configuration, paths, and required tools.

### Launch a game
```sh
sudo moma launch <game>
```

Launches the game with mods layered in via an overlay. Requires root privileges due to mounting.

## Configuration

Moma stores configuration in a single TOML file, at `~/.config/moma/config.toml`.

When you run `moma init`, it creates the file for you. You can also edit it manually to adjust paths, Proton versions, or game settings.

See `docs/config.md` for all options.

## Roadmap

### Done
✓ Initial proof-of-concept in Rust: folder structure, mount logic, and game launch support (Skyrim)

✓ Installation guide (Makefile + instructions)

### In progress
Mod installation and removal

### Planned
- (Unit) tests
- Nexus mods API integration
- Load order management
- Declarative mod installation
- NXS link handler (Nexus-style download URLs)
- FOMOD CLI wizard
- Logging for debug purposes
- Proper usage- and in depth documentation

## Motivation

There are a few reasons I started building Moma. First, I really liked how Mod Organizer 2 handled mods: no polluting the game folder, strict load order, clear separation. But running MO2 through Wine always felt clunky. I wanted something that felt native, not a Windows tool duct-taped to Linux.

I looked around. There are plenty of Linux CLI mod managers, but none of them scratched the same itch. Most still put mods directly into the game folder, rely on Wine, or depend on external tools to function. They worked, but not the way I wanted them to.

Then I discovered overlayfs, which turned out to be perfect for mod management. It lets you layer mods cleanly, isolate writes into an upper directory, and when paired with a private namespace makes the whole thing seamless and invisible to the user. No performance hit, no mess.

Finally, I thought: why not write it in a language I’ve never used? Might as well learn something from it, which is why I picked rust. Moma might not be the most feature-rich mod manager out there, or the best written one, but it’s one that felt right and that I genuinly want to use.

---

Want to contribute? Found a bug? File an issue or open a PR. I’ll probably merge it.
