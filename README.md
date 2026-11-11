# Git, three ways

Three native macOS Git browsers built with **GPUI Kit**, **Rune**, and **Expo Exact**. Each opens a repository, browses all-branch history, loads older commits, refreshes, and displays commit metadata, the full message, file statistics and unified/split diffs. All operations are read only.

## Download a Mac preview

Native `.app` downloads are published by the [macOS workflow](https://github.com/mountainforge/rune-chat/actions/workflows/macos.yml) to [GitHub Releases](https://github.com/mountainforge/rune-chat/releases). Open the DMG and drag the app to Applications; choose the architecture matching your Mac. ZIP archives are available too. These are development previews, ad-hoc signed and not notarized. macOS may require approval in **System Settings → Privacy & Security** at first launch. Git must be installed. Check each release's build status and validation notes.

## Run on a Mac

Install Xcode and select it with `xcode-select` (Exact uses the full Xcode toolchain), Git, [Rust/rustup](https://rustup.rs), and [Bun 1.4.2](https://bun.sh). Use the existing checkout; no extra Git worktree is needed. Apple Silicon is the primary intended target; Intel Mac builds need verification too.

From this repository's root:

```sh
cargo run --locked -p git-gpui
cargo run --locked -p git-rune
```

Click **Open repository** and choose a local checkout or bare Git directory. Optional for the Rust apps: `GIT_CLIENT_REPO=/path/to/repo cargo run --locked -p git-rune` opens that directory at launch. This environment variable also works with GPUI.

For Exact, bootstrap its pinned SDK once, then launch the Mac app:

```sh
./tools/setup-exact.sh
./tools/exact.sh mac --run
```

The SDK goes under ignored `.deps/exact`. The setup command downloads Exact's declared toolchains and verified runtime bundles and can take several minutes. `EXACT2=/path/to/pinned/exact ./tools/exact.sh …` uses an existing SDK instead. The app's generated launcher is run through Bun; use the wrapper so its SDK path does not depend on this development machine.

Exact uses native AppKit controls and its desktop child-process module. Its chooser is the macOS system folder chooser through `osascript`; Git and osascript must be on the app's PATH. It is a desktop app: a web build can check layout, but cannot execute Git or open local repository folders. No server, API key or GitHub login is required for any client.

## What is shown

The top pane shows a topological ancestry graph across all refs and HEAD, including unmerged branches and branch/tag labels. Select a commit to populate the lower-left changed-files list, then select a file to see its diff on the lower right. Root commits show additions; merges compare with their first parent. Binary changes and renames are represented explicitly. Commit metadata and full messages remain available in the scrollable commit summary.

The design follows the supplied [Arbor artifact](https://claude.ai/artifact/VpSMpWpzvcTJDMwJZbX6pr#play-15dd8cd198dd) in light mode: warm canvas, teal selections, curved branch lanes, author avatars, compact ref chips and colored patches. The commit summary lives above the changed files in the lower-left sidebar. GPUI uses **GPUI Kit 0.9.1**, including its buttons, resizable panes and full diff widget. Rune and Exact use the same workspace structure with custom line-numbered unified/split diff surfaces and fixed pane proportions. All three retain repository opening, refresh and pagination. There is no branch checkout or repository-writing UI.

Very large diff displays are visibly truncated after 2 MB. Git output is collected before truncation. Rust appends history pages; Exact re-reads the loaded prefix. Refresh after the repository changes externally. Chooser cancellation leaves the current repository open.

## Validate

```sh
cargo test --locked -p rune-git-core
cargo test --locked -p git-rune
cargo check --locked -p git-gpui -p git-rune
cd apps/exact-client
bun test git.test.ts
cd ../..
./tools/exact.sh contract build app.contract --json --errors-only
./tools/exact.sh test web
```

The Rust and TypeScript Git tests create real temporary repositories. A repeatable 105-commit example is available with `python3 tools/make-fixture.py /path/to/empty-folder`. The default web test checks the initial Exact interface. To exercise the populated, unchanged Contract with snapshots captured from real Git:

```sh
python3 tools/make-fixture.py /tmp/git-client-demo
bun tools/exact-ui-smoke.ts /tmp/git-client-demo
```

This tests commit selection, file switching, per-file patches, unified/split switching and pagination through a test-only data seam. It does **not** establish native Mac process integration. To test the unchanged production data module on a Mac, with real Git and a scripted folder-dialog result:

```sh
bun tools/exact-native-smoke.ts /tmp/git-client-demo
```

This checks opening, commit/file selection, actual patches, diff modes, pagination, empty/root commits and refresh through the native host. The real system folder-dialog interaction still needs manual checking. On a Mac, additionally run the manual checklist in [docs/COMPARISON.md](docs/COMPARISON.md) for each app. Source execution is provided; signed/notarized release packaging is not included.

[The comparison document](docs/COMPARISON.md) records sources, easy and difficult parts, tradeoffs, and validation evidence. macOS runtime behavior must be checked on a Mac; Linux compilation is recorded separately.

The [macOS CI workflow](.github/workflows/macos.yml) builds native bundles, runs tests, checks that the packaged Rust processes remain running with a real repository, and publishes successful previews to GitHub Releases. Build-time checks and manual interaction validation are recorded separately in the comparison document.
