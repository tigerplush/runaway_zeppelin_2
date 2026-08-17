# Implementation Roadmap

Based on the design in `runaway_zeppelin_design/` as of 2026-08-17. Built around Bevy 0.19, with `bevy_common_assets` (RON) for data-driven content, `yarnspinner` for branching narrative, `chrono` for the in-game calendar, `bevy_enhanced_input` for input, and `bevy-inspector-egui` for dev tooling.

## Guiding principle: vertical slice before breadth

Get one complete loop iteration working end-to-end early — travel to a POI, trigger a scripted Event, spend/gain resources, advance time — before building out every system in parallel. The design has a lot of interlocking systems (Lift, Morale, NPC needs, Policies); a thin working slice will surface integration problems much earlier than building each system to completion in isolation.

This is a from-scratch rewrite. The previous prototype (deleted in commit `8460918`) is intentionally not being reused — treat it only as informal prior art, not as a source to port code from.

## Phase 0 — Engine scaffolding

- App state machine (Loading / MainMenu / InGame / Paused), one plugin per module (map, zeppelin, NPCs, resources, time, events, UI).
- Design and implement the hex grid, pan-orbit camera, pointer picking, and time-speed plumbing fresh, informed by what you'd do differently this time.
- Decide RON schemas for data-driven content now: Events, POIs, Traits, Jobs, Policies. Getting this shape right early avoids reworking every system that reads it later.

## Phase 1 — World & time skeleton

- Hex map generation per [[Strange Worlds]]: sized so starting resources can't cover a full crossing, POI density tuned so a few are always visible, mountain-range obstacles gating on cruising height.
- Fog of war: revealed radius around the Zeppelin, greyed-out memory outside current sight.
- In-game clock: starts paused on 11 Oct 1928, pause/speed1(48x)/speed2/speed4 per [[Time]], backed by `chrono`.
- Zeppelin token moves across hex tiles via A* with waypoint override.

## Phase 2 — Resources & Lift

- [[Resources]] data model: water (ballast/fresh/grey), food (raw/meals/luxury/rations), fuel (liquid/solid/gaseous), materials, cargo — each with weight.
- [[Lift]]: sum(lift) − sum(weight), 0–100 valid band, fall-and-crash below 0, uncontrolled ascent + extra fuel burn above 100.
- Wire obstacle cruising-height requirement (Phase 1) into the lift system.

## Phase 3 — NPC simulation

- [[NPC]]: crew vs. passengers, per-tick needs (food/drink/rest/hygiene/socializing/entertainment/luxury/safety/health), traits, jobs/shifts.
- [[Morale]]: per-NPC tracking, mutiny trigger when enough NPCs are low.
- Death: starvation/dehydration, thrown overboard, expedition death, sickness.
- Food service scheduling: 3x/day, crew mess hall vs. passenger kitchen prep.

## Phase 4 — POI / Event / Expedition loop

- [[Point of Interest]] + [[Reach an event]]: course preview (resource/time cost), land-vs-moor per POI config.
- [[Zeppelin]] hull stress: automatic damage on landing, mitigated only by upgrades (not crew assignment — confirmed this explicitly during design review).
- [[Event]] + [[Plan and send out expedition]]: embarkation menu (assign NPCs + resources), radio reports, branching outcomes that can be referenced by later events.
- This is where `yarnspinner` earns its place — build the bridge between Yarn nodes and gameplay state (resource deltas, flags for future POIs) here, since nearly every later phase's content depends on it.

## Phase 5 — Policies & Cooking

- [[Policy]]: enact/cancel with an activation delay (cancel-while-pending is free), standing vs. one-off, resource trade-offs, temporary morale debuff after cancelling a morale-boosting policy.
- [[Cooking]]: raw food → prepared meals (costs water), raw food → rations (better exchange rate than meals), luxury goods via distillery upgrade.

## Phase 6 — Zeppelin upgrades & meta progression

- [[Upgrade zeppelin]]: cargo-space conversion (garden/hydroponics, workshop, radio station, navigation room), specialist requirements.
- [[Meta Progression Updates]]: currency on victory only (confirmed all-or-nothing during design review), cargo-sale menu, unsold cargo carries to next run, buyable zeppelin models/upgrades.
- Needs its own persistent save file, separate from any in-run state — meta-currency, unlocked upgrades, and (Phase 7) previous-run wrecks/survivors all live here.

## Phase 7 — Storms & Factions

- [[Storm]]: wandering storms placed far from the player at start, drift each tick, flying into one ends the run in victory.
- [[Strange Worlds]] factions: stranded-people settlements (reputation-gated recruitment, years-to-decades of civilization level), survivors from the player's own previous runs (persisted via the Phase 6 meta-save, may have died and left loot), occasional "wants to join the crew" storylines. Content-heavy, Yarn-driven like Phase 4.

## Phase 8 — UI & feedback

- Main UI: average morale, lift gauge, resource bars, time controls.
- Embarkation menu UI, Purser's Cabin (policy) UI, map/pathfinding UI.
- Keep `bevy-inspector-egui` dev-only (feature-gated out of release builds).

## Phase 9 — Content authoring & balancing

- Author real POI/Event content in RON + Yarn once the systems in Phases 1–7 are stable.
- Balance consumption rates, morale decay, lift math, and mutiny thresholds against the target session length (days-to-weeks in-game, a few hours real-time per the GDD).

## Suggested first milestones (small, testable, in order)

1. Empty hex map + pan-orbit camera + ticking clock; pause/speed1/2/4 all work.
2. Click a hex → Zeppelin pathfinds and moves there, resources drain per tile.
3. One hardcoded POI triggers one hardcoded Yarn Event with a branching choice that changes a resource.
4. NPCs exist, needs tick down, morale is tracked; food service consumes rations on schedule.
5. Lift computed from weight; crash below 0 works; an obstacle gates on cruising height.
6. Embarkation menu: assign NPCs to an expedition, outcome affects morale/resources.
7. A policy can be enacted from a menu with its activation delay and resource effect.
8. Meta-progression save/load: a victory grants currency, spend it on an upgrade, next run reflects it.
9. First faction/survivor content placed on the map.
