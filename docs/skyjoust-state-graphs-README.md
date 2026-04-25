# Project Skyjoust state graph bundle

This bundle contains a machine-readable state graph design for the high-level game engine flow.

Files:

- `skyjoust_state_graphs.yaml` — canonical readable spec.
- `skyjoust_state_graphs.json` — JSON equivalent of the same spec.
- `skyjoust_overview.dot` — Graphviz overview showing parallel graphs and data flow.
- `skyjoust_match_lifecycle.dot` — Graphviz match lifecycle state graph.
- `skyjoust_event_reward_scoring.dot` — Graphviz ceremony/scoring/reward state/data flow.
- `skyjoust_player_action.dot` — Graphviz per-player parallel action graph.

Suggested engine usage:

1. Treat each statechart as a small resource with its current state path.
2. Evaluate guards with pure selectors over the Bevy ECS World during fixed ticks.
3. Buffer transition actions into deterministic command queues.
4. Emit events into a stable, tick-indexed event bus.
5. Let Scoring and Rewards consume events after gameplay resolution, not during input handling.
