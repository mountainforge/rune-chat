# Exact: resume on a Mac

CI retries are paused at the user's request. Do not dispatch another run without new authorization.

The [last run](https://github.com/mountainforge/rune-chat/actions/runs/42974498122) compiled the native Apple Silicon bundle and passed the initial-screen test. The production Git interaction test failed before the folder-picker result could return:

```text
openError: Error: execFile: native response ceiling exceeds the lane's byte budget
```

The JavaScript adapter accepts a 16 MiB capture limit, but reserves an encoded reply for both stdout and stderr. The Apple executor then charges process replies at twice that ceiling. This exceeds its independent lane budget even for the tiny picker response. The source now requests **8 MiB**, which fits with headroom. The local SDK-adapter regression reproduces the rejection at 16 MiB and passes at 8 MiB. **This latest change has not been rerun on macOS.** No Exact DMG has been published.

Already fixed: SDK dependency installation order (Playwright revision mismatch), the AppKit `quinaryLabel` binding, and `Self.figures` shadowing in the SDK's frame-health diagnostics. The two Swift fixes are applied idempotently by `tools/patch-exact.py` to the pinned checkout.

## Run locally

Install full Xcode, Git, Rust/rustup and Bun 1.4.2, then use the current `main` checkout:

```sh
export EXACT_FEEDBACK=0
./tools/setup-exact.sh
bun test apps/exact-client/git.test.ts tools/exact-process.test.ts
bun tools/build-exact-macos.mjs
python3 tools/make-fixture.py /tmp/arbor-exact-mac-check
bun tools/exact-native-smoke.ts /tmp/arbor-exact-mac-check
./tools/exact.sh mac --run
```

Use a new empty fixture directory if that path already exists. Setup prepares the pinned SDK; subsequent local builds can reuse its outputs. `build-exact-macos.mjs` checks the SDK's native receipts before reusing a bundle. The native smoke test has 32 steps and uses the unchanged production data module, actual Git, and a scripted folder-dialog response. On failure it prints the native state and logs. The real system folder dialog, cancellation, and opening the user's own repository still need manual checking.

## Evidence and remaining checks

- Local: seven Git/production-data tests pass (41 assertions), including the actual SDK JS process adapter with an OS transport seam, real Git, cancellation and process errors.
- Local: Exact's web verifier is clean; populated UI tests pass for commits, files, exact patches, diff modes and pagination using the documented snapshot seam.
- Mac CI: native compilation and initial-screen test pass; production Git flow remains unverified beyond the known opening refusal. Later steps must not be counted as passed.
- Current source: 8 MiB capture fix passes the local admission regression; Mac validation is pending.
- No further CI run was started after the last failure. The final documentation/fix commit uses `[skip ci]`.
