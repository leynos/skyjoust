# Skyjoust developer's guide

This guide is for maintainers changing the Skyjoust runtime contract,
Stateright validator, trace tools, and accompanying specifications.

## 1. Normative references

The validator must stay synchronized with these source documents:

- [Product requirements](skyjoust-product-requirements.md)
- [Technical design](skyjoust-technical-design.md)
- [State graph specification](skyjoust-state-graphs.yaml)
- [Validator contract](../crates/skyjoust_stateright_validator/spec/validator_contract.yaml)
- [Architecture decision records](adr/)

## 2. Validator module structure

The `skyjoust-stateright-validator` crate keeps domain logic in small modules:

| File                    | Responsibility                                                           |
| ----------------------- | ------------------------------------------------------------------------ |
| `actions.rs`            | Domain action and small enum definitions used by traces and transitions. |
| `action_generation.rs`  | Legal action enumeration for each state during model exploration.        |
| `ceremonies.rs`         | Tournament, duel, wedding, banquet, and consequence transitions.         |
| `ledgers.rs`            | Score and reward ledger state.                                           |
| `model.rs`              | Core bounded model configuration.                                        |
| `properties.rs`         | `always` invariants and `sometimes` reachability checks.                 |
| `scoring.rs`            | Score atoms, morale changes, winner selection, and reward tallying.      |
| `serde_impls.rs`        | Serialization adapter for domain types.                                  |
| `state.rs`              | Core state snapshot, state enums, and guard helpers.                     |
| `stateright_adapter.rs` | Stateright `Model` implementation for the core model.                    |
| `trace.rs`              | Concrete JSON trace replay and validation output types.                  |
| `transitions.rs`        | Top-level transition dispatcher and gameplay/reward transitions.         |

The binary `src/bin/validate_trace.rs` is process glue. The Explorer example in
`examples/serve_explorer.rs` is diagnostic glue.

## 3. Public application programming interface

The crate root re-exports the public surface used by tests, tools, and future
runtime integration:

| Symbol                                         | Purpose                                                              |
| ---------------------------------------------- | -------------------------------------------------------------------- |
| `SkyjoustInteractionModel`                     | Configures the bounded interaction model.                            |
| `SkyState`                                     | Carries the complete model snapshot.                                 |
| `SkyAction`                                    | Represents one replayable high-level action.                         |
| `TraceValidation`                              | Reports trace replay success or failure.                             |
| `TraceFailure`                                 | Describes the first failed replay step.                              |
| `validate_trace`                               | Replays concrete actions against transition guards and invariants.   |
| State enums                                    | Expose app, match, ceremony, Warfront, objective, and reward phases. |
| `ALWAYS_PROPERTIES` and `SOMETIMES_PROPERTIES` | Expose property tables for diagnostics.                              |

Serde support is intentionally isolated in `serde_impls.rs`; domain modules do
not derive serialization traits directly.

## 4. Extending the model

Model changes should be made in this order:

1. Add the `SkyAction` variant in `actions.rs`.
2. Add action generation in `action_generation.rs` so Stateright can explore
   the new transition only from legal states.
3. Add transition handling in `transitions.rs` or the appropriate feature
   module, such as `ceremonies.rs`.
4. Add or update invariants and reachability checks in `properties.rs`.
5. Update the canonical graph bundle in `docs/skyjoust-state-graphs.yaml`,
   then run `make generate-state-graphs` to regenerate
   `docs/skyjoust-state-graphs.json`.
6. Update
   `crates/skyjoust_stateright_validator/spec/validator_contract.yaml`.
7. Add a focused unit test and, when the JSON output contract changes, an
   `insta` snapshot test.

Guard helpers should stay pure and side-effect free. Transition helpers may
mutate only the cloned destination state supplied by the caller.

## 5. Local validation

Run the full Rust gates before committing code:

```sh
make check-fmt
make check-state-graphs
make lint
make test
cargo doc --no-deps --workspace
```

Run Markdown checks after documentation changes:

```sh
make markdownlint
make nixie
git diff --check
```

The Stateright Explorer can help diagnose counterexamples:

```sh
cargo run -p skyjoust-stateright-validator --example serve_explorer
```

The trace validator can replay a fixture:

```sh
cargo run -p skyjoust-stateright-validator --bin validate_trace < crates/skyjoust_stateright_validator/traces/tournament_reward_commit.json
```

Set `SKYJOUST_VALIDATOR_DEBUG=1` during debug builds to print transition
attempts during depth-first search.
