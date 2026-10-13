# Working in this repository

This personal fork reconstructs Azeroth Survivors on the benilla engine. The supported product
is the native desktop game. Build, test and launch natively; browser bundles, web servers,
WebAssembly toolchain setup and browser verification are outside this fork's scope.

Read `docs/METHOD.md`, `docs/MAP.md` and `docs/CONTRIBUTING.md` before changing engine code.
This file defines the fork's workflow; the user's instructions take precedence.

## Layout

- `crates/warcraft-survivors/`: native game launcher and read-only original-game inspector.
- `crates/benilla-app/src/survivors/`: local survival gameplay and camera choices.
- `crates/benilla-app/src/ui_survivors.rs`: adapter to the stock FrameXML interface.
- `crates/survivors-reference/`: validation of locally imported original-game profiles.
- `crates/benilla-*`: shared native engine, formats, protocol and UI.
- `scripts/native-build.sh`, `scripts/setup.sh`, `Makefile`: native build and launch workflow.

## Commands

```bash
make build                         # fetch missing game data and build the native game
make start                         # launch the existing native release build
make native                        # build, then launch
scripts/native-build.sh             # build only, without fetching data
cargo build --locked --release     # default package: warcraft-survivors
cargo test --locked --release -p benilla-app --lib survivors::
cargo test --locked -p survivors-reference
python3 -m unittest discover -s crates/warcraft-survivors/tools -p 'test_*.py'
scripts/check.sh                    # repository checks for the current change
```

Supply an existing WoW 1.12.1 Data folder with `WOW_DATA=/path/to/Data`. Settings and imported
profiles live in the fork's `benilla-config/`, resolved through `local_state`. With Fish, source
`~/.cargo/env.fish`, not the POSIX shell's `~/.cargo/env`.

## Rules

- Keep the original installed game, its assets and its saves separate from this checkout.
- Never commit Blizzard assets, FrameXML, original executables or recovered original content.
  Read game data at runtime. Inspection output and imported profiles remain ignored and local.
- Preserve upstream ancestry through actual merges. Keep engine changes small and prefer
  adapter modules for survival-specific behavior. `docs/UPSTREAM.md` describes syncing.
- Use branches and PRs against `dumontv/azeroth-survivors-re`; main changes through PRs.
- Unattended runs use `WOW_UNATTENDED=1 WOW_NOSOUND=1`. No unattended combat probes. For camera
  and UI verification, a dev build supports `WOW_SURVIVORS_PREVIEW=1`, which disables combat.
- Run relevant native checks and report actual failures. Existing gate failures are not a reason
  to silently widen a gameplay change into unrelated formatting or engine repairs.
- Shared engine platform guards may remain from upstream; they do not add a web build or web
  verification requirement. The vendored Lua runtime is also used by the native game.
