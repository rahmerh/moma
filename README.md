<p align="center">
  <img src="assets/banner.png" alt="Moma banner" width="100%" />
</p>

Moma helps you start modding your games on Linux — fast.

It layers mods over your game’s files in a clean, temporary environment, so you launch from a single merged folder without touching your original install.

This means:

- **No pollution of your game folder**
- **Seperated mod folders for easy maintenance**
- **No need to run mod manager through Wine ever again**

Moma is linux first, it skips all the Windows-based junk and uses native linux tools. Built for terminal users who want control without spending hours setting everything up manually.

**⚠️ Warning! Moma is in early development. It will break. You’ve been warned.**

## Features

- Automatic game setup, including installation of modding tools using `moma init`
- Mount mod folders and launch game using `moma launch`

## Installation

### Requirements
- [Rust](https://www.rust-lang.org/tools/install)

### Install steps
```sh
git clone https://github.com/rahmerh/moma.git
cd moma
git checkout main
make build
sudo make install
```

*Note, if you want most current features, but potentially broken code, stay on develop*

This will copy the binary to `/usr/local/bin/moma`. You can now run moma from anywhere.

## Supported games and mod sources

Moma only supports certain games (and game versions), game platforms and mod sources. This is because I won't buy games I'm not going to play, so I can't reliably add support for it. If you want a game that isn't on the list, feel free to add an issue to request support. I'd be happy to help.

### Supported games

- Skyrim SE/AE (SKSE)

### Supported game platforms

- Steam (Proton or native)

### Supported mod sources

- [Nexus mods](https://www.nexusmods.com/)

## Usage

```sh
moma init                 # Initializes moma's folder and sets up your game(s)
moma context <game>       # Set active game
sudo moma launch <game>   # Start game
moma supported            # Lists supported game keys to be used in commands listed above
```

## Game Context

Set a temporary game context:

```sh
moma context <game>
```

This lets you skip the <game> argument in later commands (e.g., moma launch).
The context is stored in /tmp/moma_state and resets on reboot or manual change.

## Roadmap

### In progress
NXM link handler (Nexus download URLs)

### Planned
- (Unit) tests
- Standardized prints and workflows
- Load order management 
- FOMOD CLI wizard
- Game file cache management
- Proper error handling (remove anyhow crate)
- Logging for debug purposes
- Proper usage- and in depth documentation
- Live mounted preview (to show overrides and metadata)
- Load order validator and sorter

## Motivation

There are a few reasons I started building Moma. First, I really liked how Mod Organizer 2 handled mods: no polluting the game folder, strict load order, clear separation. But running MO2 through Wine always felt clunky. I wanted something that felt native, not a Windows tool duct-taped to Linux.

I looked around. There are plenty of Linux CLI mod managers, but none of them scratched the same itch. Most still put mods directly into the game folder, rely on Wine, or depend on external tools to function. They worked, but not the way I wanted them to.

Then I discovered overlayfs, which turned out to be perfect for mod management. It lets you layer mods cleanly, isolate writes into an upper directory, and when paired with a private namespace makes the whole thing seamless and invisible to the user. No performance hit, no mess.

Finally, I thought: why not write it in a language I’ve never used? Might as well learn something from it, which is why I picked rust. Moma might not be the most feature-rich mod manager out there, or the best written one, but it’s one that felt right and that I genuinely want to use.
