# Repository Guidelines

## Project Structure & Module Organization
- `docs/` holds the MkDocs site and the BeerXML specs (`docs/spec/v1.0/`, `docs/spec/v1.1/`).
- `lib/` is the Python reference implementation (schema parsing + validation).
- `rust/` is the Rust reference crate (lenient reading + strict validation), tested against `samples/`.
- `tests/` contains validation runners (currently `validate_samples.py`).
- `samples/` stores XML fixtures (`original/`, `corrected/`, `v1.1_sample.xml`).
- `scripts/` includes migration and doc-generation utilities.
- `mkdocs.yml` configures the documentation site.

## Build, Test, and Development Commands
- Validate samples (auto-detects v1.0 vs v1.1):
  ```bash
  export PYTHONPATH=$PYTHONPATH:.
  python3 tests/validate_samples.py
  ```
- Rust crate (`rust/`): format, lint and test. Tests read every file in `samples/` and snapshot it with `insta`; review changed snapshots with `cargo insta review` (or rerun with `INSTA_UPDATE=always` and inspect the diff):
  ```bash
  cd rust && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
  ```
- Migrate legacy XML to v1.1:
  ```bash
  python3 scripts/migrate_v1_to_v1.1.py <input_file_or_dir> <output_file_or_dir>
  ```
- Regenerate v1.1 spec markdown from XSD:
  ```bash
  python3 scripts/xsd_to_md.py
  ```
- Serve or build docs:
  ```bash
  mkdocs serve
  mkdocs build
  ```

## Coding Style & Naming Conventions
- Python: 4-space indentation, standard library only (no external parsing deps).
- XML/XSD: keep v1.0 stable; v1.1 uses “Clean XML” (no units, ISO dates, lowercase booleans).
- Files and directories use lowercase with underscores when needed (e.g., `validate_samples.py`).

## Testing Guidelines
- Primary test entrypoint is `tests/validate_samples.py`.
- Add new fixtures to `samples/` and ensure validation passes for both schema versions.
- If a change updates the spec or XSD, include a representative XML sample.

## Commit & Pull Request Guidelines
- Commits follow a conventional pattern like `docs: ...`, `fix: ...`, `chore: ...`.
- PRs should describe the change, link relevant issues, and include updated XSD/spec/sample XML when modifying the standard.
- For BeerXML Improvement Proposals, add `proposals/BIP-XXXX-description.md` and include a prototype XML + XSD update.

## Agent Notes
- See `CLAUDE.md` and `GEMINI.md` for tool-specific workflows and architecture details.

## Agent skills

### Issue tracker

Issues are tracked in GitHub Issues for `josefjura/beerxml` (via the `gh` CLI). See `docs/agents/issue-tracker.md`.

### Triage labels

Uses the five default labels: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `CONTEXT.md` plus `docs/adr/` at the repo root. See `docs/agents/domain.md`.
