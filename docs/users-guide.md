# Skyjoust user's guide

This guide is for operators and integrators who run the Skyjoust validator
tools against the high-level interaction contract.

## 1. Validator contract with `cargo test`

The validator contract runs the Rust unit tests, integration tests, trace
replay tests, and the bounded Stateright depth-first search (DFS) property
check. It should run before changing the state graph, action set, trace format,
or scoring rules.

```sh
cargo test -p skyjoust-stateright-validator
```

Success means the explored states satisfied the `always` invariants and reached
the expected `sometimes` examples. A failure usually points to one of three
causes:

- an action became legal from the wrong state,
- a transition changed score, rewards, or Warfront data too early,
- a reachability path disappeared after a guard change.

Worked example:

```plaintext
running 1 test
test exhaustive_high_level_interaction_contract ... ok
```

In continuous integration (CI), a failed contract run should be read as a
behavioural regression unless the graph specification and validator contract
were intentionally changed together.

## 2. Stateright Explorer

The Explorer starts an interactive browser view of the bounded interaction
model. It shows reachable states, enabled actions, invariant failures, and
counterexample paths.

```sh
cargo run -p skyjoust-stateright-validator --example serve_explorer
```

Open `http://localhost:3000/` after the process reports that the Explorer is
serving. The state graph view is useful for inspecting why a transition is
reachable. Counterexample paths list the exact action sequence that violates a
property.

Worked example:

```plaintext
attempting to serve Stateright Explorer at localhost:3000
Stateright Explorer is serving Project Skyjoust at http://localhost:3000/
```

The Explorer is a developer-facing diagnostic surface. It does not replace the
automated `cargo test` contract check.

## 3. Trace validation

The `validate_trace` binary replays a concrete JSON action array against the
same guards and invariants as the model checker. Standard input carries the
trace, standard output carries the JSON result, and standard error carries
optional diagnostics.

```sh
cargo run -p skyjoust-stateright-validator --bin validate_trace \
  < crates/skyjoust_stateright_validator/traces/keep_breach_reward_commit.json
```

The output has three top-level fields:

| Field         | Meaning                                                                |
| ------------- | ---------------------------------------------------------------------- |
| `ok`          | `true` when every action was legal and every invariant held.           |
| `final_state` | The last accepted state, or the state at the failing step.             |
| `failure`     | `null` on success, or `step_index`, `action`, and `reason` on failure. |

Exit code `0` means the trace validated. Exit code `2` means the trace was
well-formed JSON but failed the Skyjoust interaction contract.

Worked example:

```sh
printf '%s\n' '["AssetsLoaded","StartSkirmish","CommitRewards"]' \
  | cargo run -p skyjoust-stateright-validator --bin validate_trace
```

Expected result:

```json
{
  "ok": false,
  "final_state": {
    "depth": 2,
    "app": "SkirmishSetup",
    "warfront": "Inactive",
    "match_phase": "Inactive",
    "ceremony": "Dormant",
    "rules": {
      "ordnance": "Full",
      "friendly_fire": true,
      "duel_lock": false,
      "scoring_frozen": false,
      "joust_only": false,
      "allow_sudden_death": true
    },
    "player_ordnance": "Ready",
    "lance": "Idle",
    "recovery": "Alive",
    "objectives": {
      "keep_breached": false,
      "outpost_controlled": false,
      "shrine_claimed": false,
      "supply_route_blocked": false,
      "hostage_delivered": false
    },
    "score": {
      "open": false,
      "finalized": false,
      "pending_delta": false,
      "events_accepted": 0,
      "red_score": 0,
      "blue_score": 0,
      "red_glory": 0,
      "blue_glory": 0,
      "red_morale": 10,
      "blue_morale": 10,
      "victory_pending": false
    },
    "rewards": {
      "phase": "Dormant",
      "pending_delta": false,
      "committed": false,
      "glory": 0,
      "coin": 0,
      "influence": 0,
      "laurels": 0,
      "penalties": 0,
      "tournament_bonus_granted": false,
      "duel_bonus_granted": false
    },
    "winner": "None",
    "truce_active": false,
    "truce_broken": false,
    "tournament_rounds_won": 0,
    "tournament_completed": false,
    "duel_resolved": false,
    "duel_consequence_active": false,
    "treaty_signed": false,
    "infamy": 0,
    "post_final_score_write": false,
    "warfront_mutated_during_match": false
  },
  "failure": {
    "step_index": 2,
    "action": "CommitRewards",
    "reason": "action was not legal from the current state"
  }
}
```

Longer traces can raise the replay depth bound:

```sh
cargo run -p skyjoust-stateright-validator --bin validate_trace -- --max-depth 64 < trace.json
```

Verbose diagnostics print each replayed action and the final summary to
standard error:

```sh
cargo run -p skyjoust-stateright-validator --bin validate_trace -- --verbose < trace.json
```
