# Code Coverage Setup

This project uses `cargo-llvm-cov` for code coverage analysis, configured to work within institutional constraints (all
file operations restricted to `C:\Studentwork`).

## Quick Start

Run coverage with a single command:

**Windows (PowerShell):**

```powershell
.\run-coverage.ps1
```

**Linux/macOS (Bash):**

```bash
./run-coverage.sh
```

This will:

1. Install `cargo-llvm-cov` if not already installed
2. Clean any existing coverage artifacts
3. Run tests with coverage enabled (streaming feature)
4. Generate an HTML report
5. Open the report in your browser

## Configuration

The project is pre-configured to keep all coverage artifacts within the project directory:

- **Environment Variables** (`.cargo/config.toml`):
  - `TMPDIR`, `TEMP`, `TMP`: Force temp files to `target/tmp`
  - `LLVM_PROFILE_FILE`: Force coverage data to `target/coverage`
  - `CARGO_LLVM_COV_TARGET_DIR`: Keep build artifacts in `target`

- **Gitignore** (`.gitignore`):
  - `coverage/`: Coverage output directory
  - `*.profraw`: Raw profile data
  - `*.profdata`: Processed profile data
  - `llvm/`: LLVM artifacts

All paths are configured to stay within `C:\Studentwork\Dev\sids`, ensuring compliance with institutional policies.

## Usage Options

### HTML Report (Default)

**Windows:**

```powershell
.\run-coverage.ps1
# or explicitly:
.\run-coverage.ps1 -Html
```

**Linux/macOS:**

```bash
./run-coverage.sh
# or explicitly:
./run-coverage.sh --html
```

### JSON Report

**Windows:**

```powershell
.\run-coverage.ps1 -Json
```

**Linux/macOS:**

```bash
./run-coverage.sh --json
```

### LCOV Report

**Windows:**

```powershell
.\run-coverage.ps1 -Lcov
```

**Linux/macOS:**

```bash
./run-coverage.sh --lcov
```

### Clean Coverage Artifacts

**Windows:**

```powershell
.\run-coverage.ps1 -Clean
```

**Linux/macOS:**

```bash
./run-coverage.sh --clean
```

This removes:

- `target/coverage/`
- `target/tmp/*.profraw`
- `target/*.profraw`
- `target/*.profdata`
- `coverage/`

## Manual Usage

If you prefer to run `cargo-llvm-cov` directly:

```powershell
# Install (if needed)
cargo install cargo-llvm-cov

# Run with HTML output
cargo llvm-cov --features streaming --lib --output-dir target/coverage --html

# Run with text summary
cargo llvm-cov --features streaming --lib

# Clean up
cargo llvm-cov clean
```

## Troubleshooting

### Coverage files outside C:\Studentwork

If you see coverage files being created outside the allowed directory:

1. Check that `.cargo/config.toml` has the correct environment variables
2. Verify `LLVM_PROFILE_FILE` points to `C:\Studentwork\Dev\sids\target\coverage`
3. Run `.\run-coverage.ps1 -Clean` to remove any stray files

### Leftover artifacts

The automated script cleans up before and after (on failure) coverage runs. If artifacts persist:

```powershell
# Use the built-in cleanup
.\run-coverage.ps1 -Clean

# Or manually
Remove-Item -Recurse -Force target\coverage, coverage, target\*.profraw, target\*.profdata
```

### Permission errors

Ensure you have write permissions to:

- `C:\Studentwork\Dev\sids\target`
- `C:\Studentwork\Dev\sids\coverage`

## Coverage Metrics

The project currently has comprehensive test coverage:

- **Streaming module**: 36 tests
- **Actor module**: 30 tests
- **Actor System module**: 19 tests
- **Total**: 85 tests

Expected coverage:

- High coverage on `src/streaming/` (well-tested)
- High coverage on `src/actors/` (well-tested)
- Lower coverage on `src/main.rs` and `examples/` (not included in `--lib` tests)

## CI/CD Integration

To integrate with CI/CD pipelines:

```yaml
# Example for GitHub Actions or similar
- name: Run coverage
  run: |
    cargo install cargo-llvm-cov
    cargo llvm-cov --features streaming --lib --output-dir target/coverage --lcov --output-path target/coverage/lcov.info

- name: Upload coverage
  uses: codecov/codecov-action@v3
  with:
    files: target/coverage/lcov.info
```

Note: Ensure CI environment has access to the required directories or adjust paths accordingly.

## Additional Resources

- [cargo-llvm-cov documentation](https://github.com/taiki-e/cargo-llvm-cov)
- [Rust testing guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [LLVM coverage mapping](https://llvm.org/docs/CoverageMappingFormat.html)
