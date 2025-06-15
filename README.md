<p align="center">
  <img src="assets/banner.png" alt="Moma banner" width="100%" />
</p>

**Moma** is a Linux-first, CLI **mo**d **ma**nager.

It uses OverlayFS to layer mods cleanly on top of the base game folder—keeping it untouched and easily reversible. Mods stay isolated and are only merged during runtime. A single unmount resets everything.

Moma avoids Windows-native tools like Mod Organizer 2 and fully embraces native Linux features. It's designed for minimal, terminal-driven modding without sacrificing control or flexibility.

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

