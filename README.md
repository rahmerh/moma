<p align="center">
  <img src="assets/banner.png" alt="Moma banner" width="100%" />
</p>

Moma is a linux first, CLI **mo**d **ma**nager.

It uses OverlayFS to layer your mods on top of the base game folder, keeping it both clean and untouched. All mods are kept separately for easy management and merged into the mounted folder. A simple unmount will revert everything.

Moma avoids dependency on Windows-native tools like Mod Organizer 2 by leaning fully into native Linux features. It's created for a minimal, terminal mod management with all the power of other GUI managers.

## Usage

Moma is still under active development. The current version is a functional prototype in Fish, with a Rust rewrite in progress.

## Features
- Prototype in Fish: mounts mods using OverlayFS

## Roadmap
Currently working on:
- First PoC (Creates folder structure, mounts folders and launches game, written in rust, just for skyrim right now)

Future plans:
1. Mod installation/uninstallation commands
2. Nexus download handler
3. FOMOD CLI wizard
4. Much, much more...
