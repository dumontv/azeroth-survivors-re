# Local original-game reference

The fork can read a locally imported class profile. It supplies starting health and the starting
ability by class ID; the existing engine still supplies combat, upgrades, art and UI. Without a
profile the fork uses its existing defaults. This is a partial reconstruction, not a claim of
gameplay equivalence.

## Import

The read-only inspector never executes the original executable. Its static-table layout is
guarded by the SHA-256 of the investigated executable. A different build is rejected and needs
a new investigation; there is no override switch.

```fish
python crates/warcraft-survivors/tools/inspect_original.py /path/to/AzerothSurvivors
mkdir -p benilla-config
cp out/original-analysis/survivors-reference.toml benilla-config/
source ~/.cargo/env.fish
env CARGO_BUILD_JOBS=8 make wasm
make start
```

The browser loads the optional profile before initializing WebAssembly. A native dev build reads
the same profile from its own `benilla-config/`. Release builds without the `dev` feature use the
state folder beside their executable, or the `BENILLA_HOME` override. Restart after importing or
changing a profile. Browser hosting through `wenilla-realm` has no profile-loading integration.

## Evidence and limits

`out/original-analysis/evidence.json` identifies the input hash, static-table addresses, class
records, ability records, original zone order, and original game module paths. Language objects
are extracted separately with their byte offsets. All recovered content stays in ignored
directories; no executable, asset, original source code, language table or recovered profile is
committed.

The investigated ELF has relative relocations naming static Rust strings. The class table has
144-byte records; key, name and starting-ability strings occur at offsets 0, 16 and 48. The class
ID is the byte at offset 124 and starting health is the float at offset 112. A complete table is
accepted only when all nine class identities and names cross-check against the independently
embedded language object. Ability and zone tables are similarly cross-checked at strides 120
and 184. Other combat numbers are retained as raw offset-indexed values, rather than assigning
unproven meanings or applying them to the fork's different combat model. The Hunter starter
also imports Auto Shot's spell ID and basic projectile fields (damage, cooldown, range and
target count, at offsets 64, 68, 72, 76 and 84). It uses the fork's projectile implementation;
this does not reproduce the original's full attack, scaling or visual behavior.

Static text describes talents, collections, gear, companions, achievements, run modifiers and
zone unlocks. It does not establish their complete algorithms, probabilities or timing. Those
systems require separate reconstruction and behavioral verification. Class health and starting
spells also need comparison with the original picker and a fresh unequipped character before
calling visual or gameplay fidelity verified.

## Verification

The browser helper tests require Node.js; the other inspection tests use Python's standard
library. Use the desktop app's bundled Node executable if Node is not on the shell's PATH.

```fish
python -m unittest discover -s crates/warcraft-survivors/tools -p 'test_*.py'
node --test web/reference-profile.test.mjs
cargo test -p survivors-reference
cargo run -p survivors-reference --example validate -- benilla-config/survivors-reference.toml
```

To return to the fork's defaults, move `benilla-config/survivors-reference.toml` outside the state
folder, rebuild the browser bundle and restart the game. No original save or installation file
is modified by the importer.
