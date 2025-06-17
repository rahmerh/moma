<p align="center">
  <img src="assets/banner.png" alt="Moma banner" width="100%" />
</p>

**Moma** is a Linux-first, CLI **mo**d **ma**nager.

Moma automates everything you need to start modding your games on Linux. It takes care of downloading mods, setting up tools, and managing your configuration in a simple, declarative format, so your mod setup stays clean, trackable, and easy to reproduce.

To keep your game installation untouched, Moma builds a temporary environment where mods are layered only when you launch the game. Your base files remain unchanged, and mods don’t need to be installed directly into the game folder.

Moma avoids Windows-native mod management tools and fully embraces native Linux features. It's designed for minimal, terminal-driven modding without sacrificing control or flexibility.

## Installation

Make sure you have Rust installed first. Then clone the repo:

```sh
git clone https://github.com/rahmerh/moma.git
cd moma
```

If you want the latest, stable binary check out main first:

```
git checkout main

```

If you want latest (and potentially broken), stay on develop.

Run the following to install the binary system-wide:
```
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
[x] Initial proof-of-concept in Rust: folder structure, mount logic, and game launch support (Skyrim)

### In progress
Installation guide (Makefile + instructions)

### Planned
1. Mod installation and removal
2. (Unit) tests
3. Nexus mods API integration
4. Load order management
5. Declarative mod installation
6. NXS link handler (Nexus-style download URLs)
7. FOMOD CLI wizard
8. Logging for debug purposes
9. Proper usage- and in depth documentation

---

Want to contribute? Found a bug? File an issue or open a PR. I’ll probably merge it.
