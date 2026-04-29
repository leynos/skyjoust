# Repository layout

This document orients new contributors to the Skyjoust repository tree. It
describes the current structure and the responsibilities of the major paths; it
is not an exhaustive listing of every file.

## Top-level tree

The tree below is a simplified orientation sketch. See the path responsibility
notes for the conventions that govern each path.

Update this sketch when new top-level directories or major crates are added so
the contributor orientation stays synchronized with the repository layout.

```plaintext
.
|-- .github/
|-- crates/
|   `-- skyjoust_stateright_validator/
|-- docs/
|   `-- adr/
|-- ref/
|-- scripts/
|-- src/
|-- AGENTS.md
|-- Cargo.toml
|-- Makefile
`-- README.md
```

_Figure 1: Simplified repository tree for contributor orientation._

## Path responsibilities

- `AGENTS.md`: repository-wide engineering, documentation, Rust, and
  validation rules for automated contributors. Read it before making changes
  and keep edited guidance tied to the relevant `make` target or project
  document.
- `Cargo.toml`: workspace root and root package configuration. The workspace
  currently includes `.` and `crates/skyjoust_stateright_validator`. Run the
  Rust gates after workspace changes.
- `Makefile`: canonical contributor command surface for build, formatting,
  linting, tests, Markdown checks, diagram validation, and state graph
  regeneration. Prefer these targets over direct tool invocations.
- `README.md`: minimal top-level project entry point. Keep it short and put
  deeper operational or design material under `docs/`.
- `src/`: root `skyjoust` package source. This is currently a small binary
  surface while the validator crate carries most implementation detail.
- `crates/skyjoust_stateright_validator/`: high-level Stateright validator
  crate for the Skyjoust interaction contract. Treat it as the executable
  specification for match lifecycle, ceremonies, scoring, rewards, Warfront
  handoff, and trace replay.
- `crates/skyjoust_stateright_validator/src/`: validator domain modules,
  including actions, state, transitions, properties, trace replay, and
  serialization adapters. Update actions, generation, transitions, invariants,
  and trace handling together when behaviour changes.
- `crates/skyjoust_stateright_validator/src/bin/`: command-line binaries for
  the validator crate. `validate_trace.rs` is process glue around the trace
  replay application programming interface (API).
- `crates/skyjoust_stateright_validator/examples/`: diagnostic examples.
  `serve_explorer.rs` starts the Stateright Explorer for manual model
  inspection.
- `crates/skyjoust_stateright_validator/spec/`: machine-readable validator
  contract files. Keep `validator_contract.yaml` synchronized with the Rust
  model and design documents.
- `crates/skyjoust_stateright_validator/tests/`: integration tests, trace
  replay tests, and snapshot coverage. Update snapshots only when the changed
  output is the intended contract.
- `crates/skyjoust_stateright_validator/traces/`: JSON trace fixtures used by
  CLI replay and regression tests. Keep them small, representative, and tied to
  concrete validator behaviour.
- `docs/`: source of truth for product requirements, technical design, guides,
  standards, state graph references, and generated diagrams. Update the
  relevant document when requirements, architecture, workflows, or generated
  artefacts change.
- `docs/adr/`: current home for Architecture Decision Records (ADRs). This
  layout predates the style guide's canonical ADR filename convention.
- `ref/`: reference design book images that define the visual contract cited
  by the technical design. Treat these as product and art-direction inputs, not
  generated runtime assets.
- `scripts/`: repository helper scripts. Keep them small, documented, and
  aligned with `docs/scripting-standards.md`.
- `.github/`: continuous integration and dependency automation configuration.
  Keep workflow jobs aligned with local `make` targets.

## Generated artefacts and fixtures

The canonical state graph source is `docs/skyjoust-state-graphs.yaml`.
`docs/skyjoust-state-graphs.json` is generated from that YAML file and checked
in so downstream tools can consume a JSON bundle without running the generator.
After editing the YAML source, run:

```sh
make generate-state-graphs
make check-state-graphs
```

Graphviz `.dot` files under `docs/` are source files for the checked-in `.svg`
diagrams. Keep the source and rendered diagram together, and run `make nixie`
when documentation diagrams change.

Snapshot files under `crates/skyjoust_stateright_validator/tests/snapshots/`
are regression fixtures. Update them only when the new output is the intended
contract.

Trace files under `crates/skyjoust_stateright_validator/traces/` are replay
fixtures. They should remain small, representative, and tied to concrete
validator behaviour.

## Change-to-validation map

- Rust source, workspace metadata, or validator contract behaviour:
  `make check-fmt`, `make lint`, and `make test`.
- Markdown documentation: `make fmt`, `make markdownlint`, and `make nixie`.
- State graph YAML: `make generate-state-graphs`,
  `make check-state-graphs`, and `make test`.
- Graphviz diagrams or documentation diagrams: `make nixie`.
- Makefile targets: `mbake validate Makefile` and the affected `make` target.
