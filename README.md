<p align="center">
  <img src="assets/banner.png" alt="Moma banner" width="100%" />
</p>

**Moma** is a Linux-first, CLI **mo**d **ma**nager.

Moma automates everything you need to start modding your games on Linux. It takes care of downloading mods, setting up tools, and managing your configuration in a simple, declarative format, so your mod setup stays clean, trackable, and easy to reproduce.

To keep your game installation untouched, Moma builds a temporary environment where mods are layered only when you launch the game. Your base files remain unchanged, and mods don’t need to be installed directly into the game folder.

Moma avoids Windows-native mod management tools and fully embraces native Linux features. It's designed for minimal, terminal-driven modding without sacrificing control or flexibility.

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

Starts the moma game initialize wizard. Allows you to pick a game and set up the paths, configuration and minimal libraries required for you to start modding.

### Launch a game
```sh
moma launch <game>
```

Launches the game with all mods layered on the game dir. Due to having to mount certain folders, this command needs to be run as su.

## Configuration

Moma stores configuration in a single TOML file, at `~/.config/moma/config.toml`.

When you run `moma init`, it creates the file for you. You can also edit it manually to adjust paths, Proton versions, or game settings.

See `docs/config.md` for all options.

## Roadmap

### In progress
- Initial proof-of-concept in Rust: folder structure, mount logic, and game launch support (Skyrim)

### Planned
1. Installation guide (Makefile + instructions)
2. Mod installation and removal
3. (Unit) tests
4. Nexus mods API integration
5. Load order management
6. Declarative mod installation
7. Nexus download integration
8. FOMOD CLI wizard
9. Logging for debug purposes
10. Proper usage- and in depth documentation
