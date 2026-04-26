# Project Skyjoust Stateright Validator

This crate is a high-level Stateright validator for the Project Skyjoust engine
state graphs. It is not a physics simulator. It is a compact interaction
contract that proves the important handoffs between parallel statecharts:

- match lifecycle gates scoring,
- ceremony events apply and clear temporary rules,
- joust, objective, and ordnance actions become scoring atoms only when legal,
- victory pushes the match into final score export,
- rewards cannot commit before a finalized score snapshot,
- truce breaks and dishonour create penalties,
- Warfront progression resumes only after match result and reward commit.

## Run the exhaustive validator

```bash
cargo test
```

The integration test runs a bounded Stateright depth-first search (DFS) over
the high-level model and asserts all `always` and `sometimes` properties.

## Explore the model in Stateright Explorer

```bash
cargo run --example serve_explorer
```

Then open `http://localhost:3000/`.

## Validate an engine trace

```bash
cargo run --bin validate_trace < trace.json
```

The trace replayer uses the same bounded `SkyjoustInteractionModel` as the
Stateright checks. The default bound is `max_depth = 24`; raise it for longer
recorded traces:

```bash
cargo run --bin validate_trace -- --max-depth 64 < trace.json
```

Example trace JSON:

```json
[
  "AssetsLoaded",
  "StartSkirmish",
  "StartBattle",
  "FinishConstructing",
  "SpawnReady",
  "CountdownDone",
  "BombKeepBreach",
  "VictoryCheck",
  "ExportFinalScore",
  "TallyRewards",
  "CommitRewards"
]
```

For enum variants with fields:

```json
{ "Joust": { "winner": "Red", "outcome": "Unhorse" } }
```

## Files

- `src/lib.rs` exports the Stateright `Model`, state definitions, invariants,
  reachability properties, and trace validator from focused modules.
- `tests/stateright_contract.rs` is the continuous integration (CI) smoke
  model check.
- `examples/serve_explorer.rs` launches Stateright Explorer.
- `src/bin/validate_trace.rs` replays a JSON action log from stdin.
- `spec/validator_contract.yaml` is the machine-readable summary of the
  validator contract.

## Extending the model

Add new gameplay interactions by doing four things:

1. Add a `SkyAction` variant.
2. Gate it in `actions` and `next_state`.
3. Add at least one invariant or reachability property if the action touches
   rules, scoring, rewards, or Warfront state.
4. Update the canonical graph bundle (`docs/skyjoust-state-graphs.*`) and
   `spec/validator_contract.yaml` whenever the new action or guard changes the
   behaviour represented by `SkyAction`, `actions`, `next_state`, or the
   invariants.

Keep the validator smaller than the runtime. The sweet spot is a model that
catches illegal handoffs without trying to simulate every mount flap and pixel
collision. Let the runtime do feathers; let Stateright watch the treaty table.
