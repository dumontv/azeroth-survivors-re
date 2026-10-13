<img width="1920" height="1080" alt="brag" src="https://github.com/user-attachments/assets/fa574bb6-7bd4-4e78-8b72-14c58e3c716e" />

<div align="center">

# Azeroth Survivors

**A free fan-made WoW roguelike by [Aethus](https://wow.aethus.pl/), run locally on a
from-scratch WoW 1.12.1 client engine (Rust + [Bevy](https://bevy.org)).**

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue.svg)](LICENSE-MIT)

</div>

> **Not an original project.** *Azeroth Survivors* is a free, fan-made WoW roguelike by
> **Aethus** — play the original in the browser at **[wow.aethus.pl](https://wow.aethus.pl/)**
> ([r/wowservers announcement](https://www.reddit.com/r/wowservers/comments/1x0yidl/azeroth_survivors_roguelike_wow_singleplayer_game/)
> · [Discord](https://discord.gg/UzxaZuSAk)). This repository is a from-source local build of the
> game, running natively on the [benilla](#how-its-built) engine, for personal use.

Pick a class and a battleground, then survive: WASD to move, your abilities cast themselves,
enemies swarm in waves, you level up and pick upgrades, and a boss shows up every couple of
minutes. **No server, no login, no account** — fully offline and self-contained.

<table>
  <tr>
    <td width="50%"><img src="docs/screenshots/menu.jpg" alt="The pick screen: race, gender and class on the left, sixteen zones on the right, the hero in between"></td>
    <td width="50%"><img src="docs/screenshots/combat.jpg" alt="A Mage holding off Elwynn Forest's wolves with frost, fire and arcane"></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/level-up.jpg" alt="Level up: three spell cards, coloured by rarity, each showing exactly what its next rank changes"></td>
    <td><img src="docs/screenshots/results.jpg" alt="The results screen after a death in Redridge Mountains"></td>
  </tr>
</table>

### In this build

- **A character-create pick screen** built from the client's own glue art: eight races, both
  genders, every class the race can be, and **sixteen zones**, from the starting zones to
  Naxxramas and the Gates of Ahn'Qiraj. Each zone fights with its own creatures and bosses, taken
  from the zone's real spawns. Your pick is remembered across reloads.
- **Level-up cards** that deal in, tilt toward the cursor and lift on hover. Their WoW item-quality
  colour says how strong a pick is, and each card lists exactly what changes, like
  `Damage 20 » 26` or `Targets 2 » 3`.
- **A class movement spell on F** from level 5 (cursor-aimed in top-down): Blink, Charge, Sprint, Dash,
  Fade, Ghost Wolf, Aspect of the Cheetah, Blessing of Freedom, Howl of Terror.
- **Third-person by default**, with the original WoW interface and movement controller.
  **F6**, or the view button beside the Escape menu, switches to the original top-down view.
  Your view choice is remembered across runs.
- A light sharpen-and-colour pass over the picture (**G** toggles it).

## Play it

```bash
git clone https://github.com/dumontv/azeroth-survivors-re
cd azeroth-survivors-re
make build     # fetch missing game data and build the native desktop game
make start     # launch the desktop game
```

`make native` builds and launches in one command. `scripts/setup.sh` also prepares a fresh
clone. Plain `cargo build --release` builds the native game, the workspace's default package.
For code changes when the game data is already present, run `scripts/native-build.sh`.

**Prerequisites:** Rust (rustup uses `rust-toolchain.toml`), a C compiler, `pkg-config`, `make`,
and `curl`/`unzip` for the optional game-data download. Linux also needs ALSA, udev, Wayland,
and xkbcommon development libraries (`libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev`
on Debian/Ubuntu). In Fish, load a new rustup installation with `source ~/.cargo/env.fish`.

Use existing game data without downloading it:

```bash
make build WOW_DATA=/path/to/WoW/Data
make start WOW_DATA=/path/to/WoW/Data
```

`make clean` removes Rust build output and preserves game data and local settings.
Run `make help` to list every target.

## Controls

See [CONTROLS.md](CONTROLS.md). The short version: **W/S** to move, **A/D** to turn,
**Q/E** to strafe, **right mouse drag** to turn with the camera, and **Space** to jump.
**F6** switches views, the **mouse wheel** zooms, **Esc** opens the pause menu,
**F** uses your class movement spell, and **1/2/3** picks a level-up card.

## The game data (not in this repo)

This repo ships **no Blizzard content** — no art, models, sounds, maps, MPQ contents, or
FrameXML. The game reads a WoW 1.12.1 client's `Data` folder at runtime, which you supply.

`make build` (via `scripts/fetch-data.sh`) downloads the **English WoW 1.12.1 (build 5875)**
client from [archive.org](https://archive.org/details/WoW_1.12.1-5875) (~5.5 GB) and extracts the
MPQs into `wow-data/Data/` (git-ignored, resumable, skipped when already present). Or point at a
client you already have with `WOW_DATA`.

### Changing the download source

If the archive.org item moves or you have your own mirror, override the URL without editing
anything:

```bash
make build WOW_DATA_URL=https://your-mirror.example/client.zip
```

Or set it permanently by editing the single clearly-marked `DEFAULT_DATA_URL` line at the top of
`scripts/fetch-data.sh`. Either way the URL must be a direct link to a zip holding the client's
`Data/*.MPQ` files.

## How it's built

The native launcher is `crates/warcraft-survivors/`; survival gameplay lives in
`crates/benilla-app/src/survivors/`. The engine is based on
[benilla](https://github.com/samwhosung/benilla), through the
[open-source reconstruction](https://github.com/OutThisLife/azeroth-survivors-re) and its wenilla
ancestry. This fork builds and tests the native desktop game only.

## Legal

*Azeroth Survivors* is a free fan game by **Aethus**, not affiliated with or endorsed by
Blizzard Entertainment, and it ships no Blizzard game files. The screenshots in `docs/screenshots/`
show World of Warcraft art rendered from a user-supplied client; that art is © Blizzard
Entertainment. World of Warcraft is a trademark of Blizzard Entertainment, Inc. The engine code in
this repository is licensed under
[MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE), at your option; the vendored components under
`third_party/` keep their own upstream licenses.
