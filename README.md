<p align="center">
  <img src="assets/banner.png" alt="Moma banner" width="100%" />
</p>

Moma is a linux first, CLI **mo**d **ma**nager.

It uses OverlayFS to layer your mods on top of the base game folder, keeping it both clean and untouched. All mods are kept separately for easy management and merged into the mounted folder. A simple unmount will revert everything.

Moma avoids dependency on Windows-native tools like Mod Organizer 2 by leaning fully into native Linux features. It's created for a minimal, terminal mod management with all the power of other GUI managers.

## Usage

Probably shouldn't right now, first fish script is there to test the concept, currently writing it in rust.

## Features
- Automatic SKSE installation and integration
- Proton-compatible launching with custom wineprefix

## Roadmap
1. First PoC (Creates folder structure, mounts folders and launches game, written in rust)
2. Mod installation/uninstallation commands
3. Nexus download handler
4. FOMOD CLI wizard
5. Much, much more...
