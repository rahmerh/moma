<p align="center">
  <img src="assets/banner.png" alt="Moma banner" width="100%" />
</p>

[![Build Status](https://github.com/rahmerh/moma/actions/workflows/ci.yml/badge.svg)](https://github.com/rahmerh/moma/actions/workflows/ci.yml)

Moma helps you start modding your games on Linux — fast.

It layers mods over your game’s files in a clean, temporary environment, so you launch from a single merged folder without touching your original install.

This means:

- **No pollution of your game folder**
- **Seperated mod folders for easy maintenance**
- **No need to run a mod manager through Wine ever again**

Moma is linux first, it skips all the Windows-based junk and uses native linux tools. Built for terminal users who want control without spending hours setting everything up manually.

**⚠️ Warning! Moma is in early development. It will break. You’ve been warned.**

## Features

- **One-command setup** – Automatically prepares your modding environment and config folders.
- **Clean modded game launching** – Keep your base game untouched with isolated mod layers.
- **Mod source integration** – Connect to sources like Nexus Mods with secure API key handling.
- **Live download tracking** – Monitor active downloads with real-time progress and status.

## Installation

### Requirements
- [Rust](https://www.rust-lang.org/tools/install)

### Install
```sh
git clone https://github.com/rahmerh/moma.git
cd moma
git checkout main
make build
sudo make install
```
> *Want the latest features (and bugs)? Stay on develop.*

This installs `moma` to `/usr/local/bin`, so you can run it from anywhere.

## Supported games and mod sources

Moma only supports certain games (and game versions), game platforms and mod sources. This is because I won't buy games I'm not going to play, so I can't reliably add support for it. If you want a game that isn't on the list, feel free to add an issue to request support. I'd be happy to help.

### Supported games

- Skyrim SE/AE (SKSE)

### Supported game platforms

- Steam

### Supported mod sources

- [Nexus mods](https://www.nexusmods.com/)

## Usage

```sh
moma init                 # Initializes moma's folders and sets up your game(s)
moma context <game>       # Set active game context
sudo moma launch <game>   # Start game with your mods

moma connect <source>     # Connects to the mod source, prompting and storing API keys if required

moma mod install          # Opens a menu to install downloaded mods into your game
moma mod downloads        # Displays all active downloads and progress
```

## Game Context

You can set a temporary game context:
```sh
moma context <game>
```

This lets you skip the <game> argument in later commands (e.g., moma launch).
Stored in `/tmp/moma_state`, resets on reboot or when changed manually.

## Roadmap

### In progress
NXM link handler (Nexus download URLs)

### Planned
- (Unit) tests
- Persist, warn and handle sink directory (mod config files/runtime writes)
- Automatic read and validation of extracted mod archives
- Standardized prints and workflows
- TUI
- Load order management 
- FOMOD CLI wizard
- Game file cache management
- Proper error handling (remove anyhow crate)
- Logging for debug purposes
- Proper usage- and in depth documentation
- Websocket server for download (or others) progress
- Live mounted preview (to show overrides and metadata)
- Load order validation and sorting

## Releases
These aren't proper releases, more so milestones I feel comfortable pushing to main.

### 0.4.0
- Mod source "connection" setup with `moma connect <source>`
- Mod downloads/installation management with `moma mod downloads/install`

### 0.3.0
- Automated install with make

### 0.2.0
- Overlay mount and game launch with `sudo moma launch`

### 0.1.0
- Automatic game setup with `moma init`

## Motivation

There are a few reasons I started building Moma. First, I really liked how Mod Organizer 2 handled mods: no polluting the game folder, strict load order, clear separation. But running MO2 through Wine always felt clunky. I wanted something that felt native, not a Windows tool duct-taped to Linux.

I looked around. There are plenty of Linux CLI mod managers, but none of them scratched the same itch. Most still put mods directly into the game folder, rely on Wine, or depend on external tools to function. They worked, but not the way I wanted them to.

Then I discovered overlayfs, which turned out to be perfect for mod management. It lets you layer mods cleanly, isolate writes into an upper directory, and when paired with a private namespace makes the whole thing seamless and invisible to the user. No performance hit, no mess.

Finally, I thought: why not write it in a language I’ve never used? Might as well learn something from it, which is why I picked rust. Moma might not be the most feature-rich mod manager out there, or the best written one, but it’s one that felt right and that I genuinely want to use.
