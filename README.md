<p align="center">
  <img src="assets/banner.png" alt="Moma banner" width="100%" />
</p>

**Moma** is a Linux-first, CLI **mo**d **ma**nager.

It uses OverlayFS to layer mods cleanly on top of the base game folder—keeping it untouched and easily reversible. Mods stay isolated and are only merged during runtime. A single unmount resets everything.

Moma avoids Windows-native tools like Mod Organizer 2 and fully embraces native Linux features. It's designed for minimal, terminal-driven modding without sacrificing control or flexibility.

---

## Usage

Moma is under active development. The current prototype is a Fish script. A full Rust rewrite is in progress.

---

## Roadmap

### In progress
- Initial proof-of-concept in Rust: folder structure, mount logic, and game launch support (Skyrim)

### Planned
1. Mod installation and removal
2. Nexus download integration
3. FOMOD CLI wizard

