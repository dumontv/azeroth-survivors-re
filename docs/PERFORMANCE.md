# Native performance

Use the same device, resolution, camera position and scene when comparing builds. Measure cold
and warm startup separately, then frame p50/p95/p99, memory and stalls during terrain crossings.
A successful build or screenshot does not establish an FPS improvement.

## Builds and diagnostics

- `scripts/native-build.sh` builds the native release game used for comparisons.
- `cargo survivors` runs the native game with the incremental `play` profile for iteration.
- `cargo build --locked --profile ship -p warcraft-survivors` enables the existing fat-LTO
  shipping profile for an explicit comparison. Release remains the development build default.
- `/console fpsJournal 1` records to `benilla-config/Diagnostics/fps-journal.csv` in a dev build.
  `WOW_FPS_JOURNAL=/path/to/report.csv` selects an explicit output path. State paths resolve
  through `local_state`; keep reports outside the read-only game-data folder.

## Shared engine code

The retained non-macOS rig-composition scratch buffers and UI event-listener lookup are covered
by differential and mutation tests. When syncing upstream, compare the original math and event
ordering, and keep the engine's native collision and audio paths working.

The historical September 2026 review describes the parent project's browser and hosting stack.
Those applications and their build workflow are removed from this fork. Its performance claims
are not measurements of this native survival game.

Use the procedure in `UPSTREAM.md` for engine updates. Unattended diagnostics must be marked
`WOW_UNATTENDED=1 WOW_NOSOUND=1`; do not run unattended combat probes. Camera and UI checks can
use the developer-only `WOW_SURVIVORS_PREVIEW=1` non-combat scene.
