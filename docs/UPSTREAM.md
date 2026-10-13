# Syncing upstream

This checkout's `origin` is `dumontv/azeroth-survivors-re` and `upstream` is
`OutThisLife/azeroth-survivors-re`. The shared engine originates in
[samwhosung/benilla](https://github.com/samwhosung/benilla); fetch that repository through a
separately named remote if an engine sync is intended.

The product built by this fork is the native desktop game. Keep the native launcher, survival
reconstruction and native build workflow when resolving upstream changes. Browser entry points,
web servers and web build jobs removed from this fork should not be restored by a sync.

## Procedure

Start with a clean working tree and inspect the upstream changes before merging:

```bash
git fetch origin
git fetch upstream
git switch -c sync-upstream origin/main
git log --oneline HEAD..upstream/main
git merge upstream/main
```

Resolve conflicts in small pieces, preserving the original merge ancestry. Shared engine
platform guards can remain when they do not affect the native build. Keep reconstruction code
in its adapter modules and preserve the stock UI loading from user-supplied game data.

Verify on the native target:

```bash
scripts/native-build.sh
cargo test --locked --release -p benilla-app --lib survivors::
cargo test --locked -p survivors-reference
scripts/check.sh
```

For visible camera or UI changes, verify a native run. Unattended runs must be silent and
explicitly marked, and must not simulate combat (`docs/METHOD.md`). A non-combat survival preview
is available in dev builds through `WOW_SURVIVORS_PREVIEW=1`.

Open the sync PR against the personal fork:

```bash
gh pr create -R dumontv/azeroth-survivors-re --base main
```

Merge an upstream sync with a merge commit. Squashing a sync loses its ancestry and causes later
merges to replay already resolved upstream changes. Ordinary feature PRs may be squashed.
Never discard local changes through GitHub's Sync fork button, force-push main, or merge
unrelated histories without investigating the repository's actual merge base.
