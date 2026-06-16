# cargo-soteria

A Cargo subcommand for running [soteria-rust](https://github.com/soteria-tools/soteria) analysis on Rust projects.

## Overview

`cargo-soteria` provides a convenient way to run soteria symbolic execution on your Rust crates. It ships with pre-built binaries of soteria-rust and all its dependencies, so you don't need to install OCaml or build the tool from source.

## Installation

```bash
cargo install soteria
cargo soteria setup # Installs binaries
```

The setup installs the latest nightly release (~27MB compressed, ~85MB uncompressed) with all necessary tools.

## Uninstallation

To uninstall (including extracted packages in `~/.soteria/`):

```bash
cargo soteria unsetup
cargo uninstall soteria
```

The `soteria-cleanup` utility will show you what will be removed and ask for confirmation before deleting anything.

## Usage

Run soteria analysis on your crate:

```bash
cd your-rust-project/
cargo soteria --kani
```

`cargo soteria` discovers every symbolic test in the crate and analyses them in
**parallel** — one `soteria-rust` process per test — streaming each result as it
finishes. All other arguments (e.g. `--kani`) are forwarded to the workers.

Use `-j`/`--jobs` to control how many tests run at once (default: a quarter of
the available CPUs), and press Ctrl-C to stop — every running analysis is killed:

```bash
cargo soteria -j 8 --kani
```

### Common Options

- `-j, --jobs <N>` — Number of tests to analyse concurrently (default: CPUs / 4)
- `--kani` — Use Kani verification harnesses
- `--filter=<pattern>` — Only analyse tests whose name matches the regex

See `cargo soteria --help` for all available options.

## Example Test

Create a simple verification test using the Soteria API:

```rust
// src/lib.rs

#[soteria::test]
fn verify_addition() {
    let a: u32 = soteria::nondet_bytes();
    let b: u32 = soteria::nondet_bytes();
    
    soteria::assume(a < 1000);
    soteria::assume(b < 1000);
    
    let result = a + b;
    assert!(result >= a);
    assert!(result >= b);
}
```

Soteria can also run existing Kani harnesses using:
```
cargo soteria --kani
```

Run the analysis:

```bash
cargo soteria
```

Output:
```
  Soteria running 1 test · 1 worker

  ✓ verify_addition  0.21s

  ── Summary ──────────────────────────
  1 passed   in 0.2s
```

When a test fails or the analyzer crashes, its diagnostics are printed inline
under the result, and the run finishes with a list of the failing tests and a
non-zero exit code (`1` if any failed, `2` if any crashed, `130` if interrupted).

## Docker

A pre-built image is published to GitHub Container Registry with soteria-rust
already baked in — no `cargo install`, no `setup` step:

```bash
docker run --rm -v "$PWD:/workspace" \
  ghcr.io/soteria-tools/cargo-soteria:nightly --kani
```

The crate in the current directory is mounted at `/workspace` (the image's
working directory). Any arguments after the image name are forwarded to
soteria-rust, exactly like `cargo soteria <args>`.

**Tags:**

- `:nightly` — moving tag, rebuilt daily with that day's soteria-rust nightly.
- `:YYYY-MM-DD` — immutable per-day tag, for pinning or rolling back. A rolling
  ~7-day window of dated tags is retained.

**Runs as a non-root user.** The container runs as the unprivileged `soteria`
user, so build artifacts written into your mounted directory (`target/`,
`Cargo.lock`, `*.llbc.json`, …) are **not** owned by root. If you need them
owned by your host user specifically, pass your UID/GID:

```bash
docker run --rm -u "$(id -u):$(id -g)" -v "$PWD:/workspace" \
  ghcr.io/soteria-tools/cargo-soteria:nightly --kani
```

**Platform:** the image is `linux/amd64` only. On Apple Silicon / other hosts
it runs under emulation (Docker pulls the amd64 image automatically).

## Architecture Support

Currently supported platforms:
- **macOS ARM64** (Apple Silicon)

The package structure allows easy addition of more platforms. Each architecture-specific package is built separately to keep binaries small and installation fast.

## How It Works

1. **Build time**: The build script (`build.rs`) detects your target OS/architecture and compresses the appropriate pre-built binary package into a tar.gz archive.

2. **Compile time**: The archive is embedded directly into the `cargo-soteria` binary using `include_bytes!`.

3. **First run**: On first execution, the binary extracts the package to `~/.soteria/<version>/` and sets executable permissions.

4. **Every run**: The binary sets up the required environment variables, discovers the crate's tests (`soteria-rust compile --list-tests`), and analyses them in parallel — one `soteria-rust exec` process per test, with results streamed as they finish (see `src/run.rs`).

## Environment Variables

The following variables are automatically set when running soteria-rust:

- `DYLD_LIBRARY_PATH` (macOS) / `LD_LIBRARY_PATH` (Linux) — Points to bundled dynamic libraries
- `SOTERIA_Z3_PATH` — Path to the Z3 SMT solver
- `SOTERIA_OBOL_PATH` — Path to the Obol frontend
- `SOTERIA_CHARON_PATH` — Path to the Charon frontend  
- `SOTERIA_RUST_PLUGINS` — Path to verification plugins (kani, miri, rusteria)

You can override these if needed, but the defaults work out of the box.

## Project Structure

```
cargo-soteria/
├── Cargo.toml          # Package metadata and dependencies
├── build.rs            # Compresses platform-specific packages at build time
├── packages/           # Pre-built binaries per platform
│   └── macos/
│       └── aarch64/
│           ├── bin/    # soteria-rust, z3, obol, charon, etc.
│           ├── lib/    # Dynamic libraries (libgmp, etc.)
│           └── plugins/# Verification API crates (kani, miri, rusteria)
└── src/
    ├── main.rs         # CLI dispatch, setup/unsetup, install, env setup
    ├── run.rs          # Parallel test runner (-j/--jobs, Ctrl-C teardown)
    ├── help.rs         # `cargo soteria --help` rendering
    └── cleanup.rs      # the `soteria-cleanup` binary
```

## Adding New Architectures

To add support for a new platform (e.g., Linux x86_64):

1. Download the soteria-rust package from CI:
   ```bash
   cd cargo-soteria
   gh run list --repo soteria-tools/soteria -b main --limit 5
   gh run download <run_id> \
     --repo soteria-tools/soteria \
     --name "ubuntu-latest-soteria-rust-package" \
     -D packages/linux/x86_64
   ```

2. Build for that target:
   ```bash
   cargo build --target x86_64-unknown-linux-gnu --release
   ```

3. Test:
   ```bash
   cargo install --path . --target x86_64-unknown-linux-gnu
   ```

The build script automatically detects the target platform and selects the appropriate package.

## Limitations

- Currently only supports macOS ARM64
- The binary is large (~27MB) due to embedded toolchain
- First run requires ~85MB disk space for extraction

## Updating Packages

The soteria-rust packages are automatically updated daily via GitHub Actions. When a new version is available from the upstream [soteria repository](https://github.com/soteria-tools/soteria), it is automatically committed to main.

Package versions are tracked using `.package-version.json` files that contain the CI run ID and commit SHA. Updates are only performed if a newer version is available.

### Automatic Updates

The [Update Soteria Packages workflow](.github/workflows/update-packages.yml) runs daily at midnight UTC and:
1. Checks the current package version against the latest soteria CI run
2. If an update is available, downloads the macOS ARM64 package artifact
3. Creates a `.package-version.json` file with version metadata
4. Verifies the package contents
5. Commits and pushes changes directly to main

The workflow is skipped if the packages are already up to date.

You can also trigger this workflow manually:
```bash
./scripts/auto-update-packages.sh
```

### Manual Updates

To manually update the packages, use the provided Python script:

```bash
./scripts/local-update-packages.py
```

This script will:
1. Check the current package version against the latest soteria CI run
2. Exit early if packages are already up to date
3. Prompt you to confirm the download if an update is available
4. Download and extract the package to `packages/macos/aarch64/`
5. Create a `.package-version.json` file with version metadata
6. Verify the package structure and contents
7. Show a summary with next steps

**Requirements for CI download mode:**
- GitHub CLI (`gh`) installed and authenticated
- Python 3.7+

**Building from a local soteria checkout:**

If you have a local checkout of the soteria repository and want to build packages from there instead of downloading from CI:

```bash
./scripts/local-update-packages.py --from-dir /path/to/soteria
```

This will:
1. Run `make package-soteria-rust` in the specified directory
2. Copy the built package to `packages/macos/aarch64/`
3. Extract git commit metadata from the local repository
4. Create a `.package-version.json` file (without a CI run ID)
5. Verify the package structure

This is useful for testing local changes to soteria before they're merged upstream.

**After updating:**
1. Test the build: `cargo build --release`
2. Verify functionality: `cargo install --path .` and test with a sample project
3. Commit the changes: `git add packages/ && git commit -m "Update soteria packages"`

## Support

For issues related to:
- **cargo-soteria installation/usage**: Open an issue in this repository
- **soteria-rust analysis**: See [soteria documentation](https://github.com/soteria-tools/soteria)
- **Verification harness APIs**: See [soteria test examples](https://github.com/soteria-tools/soteria/tree/main/soteria-rust/test/cram)

## License

Apache-2.0

## Related Projects

- [soteria](https://github.com/soteria-tools/soteria) — The main soteria verification framework
- [Kani](https://github.com/model-checking/kani) — Rust verification tool (API compatibility)
- [Miri](https://github.com/rust-lang/miri) — Rust interpreter (API compatibility)

