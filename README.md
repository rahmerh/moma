<p align="center">
  <img src="assets/banner.png" alt="Moma banner" width="100%" />
</p>

**Moma** is a Linux-first, CLI **mo**d **ma**nager.

Moma automates everything you need to start modding your games on Linux. It takes care of downloading mods, setting up tools, and managing your configuration in a simple, declarative format—so your mod setup stays clean, trackable, and easy to reproduce.

To keep your game installation untouched, Moma builds a temporary environment where mods are layered only when you launch the game. Your base files remain unchanged, and mods don’t need to be installed directly into the game folder.

Moma avoids Windows-native mod management tools and fully embraces native Linux features. It's designed for minimal, terminal-driven modding without sacrificing control or flexibility.

## Supported games

- Skyrim (SE and AE)

## Usage

### Initialize
```sh
moma init
```

Starts the moma game initialize wizard. Allows you to pick a game and set up the paths, configuration and minimal libraries required for you to start modding.

## Roadmap

### In progress
- Initial proof-of-concept in Rust: folder structure, mount logic, and game launch support (Skyrim)

### Planned
1. Installation guide (Makefile + instructions)
2. Mod installation and removal
3. Declarative mod installation
4. Nexus download integration
5. FOMOD CLI wizard

