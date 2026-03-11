# Real Micro Machines - Implementation Plan

## Context

Build a **Micro Machines-style 3D top-down arcade racer** that generates tracks from real-world road data. Users pick a location, the game fetches OpenStreetMap data, generates a racing circuit or sprint route, and lets players race with power-ups and weapons.

**Core product thesis**: Real-world road networks can be transformed into fun, readable arcade racing tracks.

**Biggest existential risk**: Generated tracks might not be fun often enough. This must be validated before investing in multiplayer, Discord, or polish.

## Design Principles

1. **Prove the fantasy first** - Can a real-world location become a fun race? Answer this before anything else.
2. **Gameplay over cartographic purity** - Simplify intersections, widen narrow roads, suppress visual clutter. The map should *feel* real, not *be* accurate.
3. **Fast time-to-fun** - Minimize clicks and waiting before the first race. Featured locations for instant play.
4. **Graceful degradation** - Not every location works. Reject weak candidates, recommend better nearby areas, never pretend bad tracks are good.
5. **One platform at a time** - Standalone web first. Discord Activity second, once core loop is stable.

## Decisions Summary

| Decision | Choice |
|----------|--------|
| Perspective | 3D with top-down camera |
| Art style | Low-poly stylized |
| Platform | Standalone web first, Discord Activity later |
| Players | Single-player vertical slice first, then multiplayer (2-8) |
| Physics | Full arcade (drifty, bouncy) |
| Items | Power-ups and weapons (after core loop proven) |
| Map selection | Search + interactive map |
| Game modes | Circuit, Sprint, Elimination, Time Trial |
| Vehicles | 1 car MVP (color variants), architecture for 6+ |
| Frontend | TypeScript + Three.js + Vite |
| Backend | Rust (Axum + tokio-tungstenite) |
| Testing | Vitest + Playwright (E2E) + cargo test |
| Git | Short-lived feature branches off develop (lightweight) |
| Versioning | SemVer |
| Local dev | Docker Compose |
| Tracking | GitHub Projects board |
| Elevation | Deferred - flat/simplified terrain for MVP |

---

## Git Strategy (Lightweight)

```
main            ← stable releases only, tagged with SemVer
  ↑
develop         ← integration branch, always buildable
  ↑
feature/M1-*    ← short-lived feature branches, merge to develop quickly
story/task-*    ← individual units of work, branch off feature if needed
release/x.x.x  ← cut from develop when stabilizing a release
```

**Rules**:
- Feature branches are short-lived (days, not weeks). Merge to `develop` as soon as tests pass.
- Story branches off features only when the feature has multiple concurrent workers.
- Release branches only when stabilizing for a tagged release.
- No long-lived milestone branches - milestones are tracked in GitHub Projects, not branch topology.
- Refactoring and cross-cutting changes go directly on feature branches.

**Pre-commit hooks** (via lefthook):
- Client: `vitest run --changed`, `tsc --noEmit`, `eslint`
- Server: `cargo test`, `cargo clippy`, `cargo fmt --check`

---

## Milestones (Reordered for Earliest Concept Validation)

### Phase A: Vertical Slice (PROVE THE CONCEPT)

| Milestone | Version | Description | Go/No-Go Gate |
|-----------|---------|-------------|---------------|
| M1 | v0.1.0 | Project scaffolding, Docker, test infra | Builds and runs |
| M2 | v0.2.0 | Map pipeline: fetch OSM, generate crude 3D roads + buildings | Real location renders in 3D |
| M3 | v0.3.0 | Track generation: find loops/sprints, place checkpoints | >60% of test locations produce a raceable route |
| M4 | v0.4.0 | Core engine + physics: drive a car on a generated map | **"Is it fun to drive here?"** - playtest verdict |

**M4 is the concept validation gate.** If driving on generated maps isn't fun at M4, stop and rethink before building multiplayer.

### Phase B: Core Game

| Milestone | Version | Description | Go/No-Go Gate |
|-----------|---------|-------------|---------------|
| M5 | v0.5.0 | Single-player race: countdown, laps, checkpoints, finish | Complete solo race on generated track |
| M6 | v0.6.0 | Multiplayer: Rust WS server, rooms, state sync, prediction | Two tabs racing each other smoothly |
| M7 | v0.7.0 | Game modes: Circuit, Sprint, Elimination, Time Trial | All modes playable |
| M8 | v0.8.0 | Power-ups and weapons | Items affect gameplay, feel fair on generated tracks |

### Phase C: Platform & Polish

| Milestone | Version | Description | Go/No-Go Gate |
|-----------|---------|-------------|---------------|
| M9 | v0.9.0 | UI: map selector, lobby, HUD, results | Full flow works end-to-end |
| M10 | v0.10.0 | Discord Activity integration | Launches in Discord, OAuth works |
| M11 | v0.11.0 | Polish: audio, particles, LOD, performance | 60fps on mid-range hardware |
| M12 | v1.0.0 | Production release | All KPIs met |

---

## Concept Validation KPIs (measured at M4 gate)

| Metric | Target | How to Measure |
|--------|--------|----------------|
| Track generation success rate | >60% of locations produce a raceable route | Test 50 diverse locations, count successes |
| Track quality rating | >40% rated "fun" by internal playtest | Drive each generated track, rate 1-5 |
| Time from location pick to driving | <15 seconds (cached), <30 seconds (cold) | Stopwatch test |
| Top-down readability | Can anticipate turns 2+ seconds ahead | Playtest at target speed |
| Driving feel on real geometry | "Feels good" verdict from 3+ playtests | Subjective but explicit |
| Car stays on road at reasonable speeds | No geometry-caused physics glitches | Drive 10 laps on 5 different tracks |

**If these targets are not met**, the remediation path is:
1. Increase road distortion (wider roads, simplified intersections)
2. Tighten location recommendations (only offer high-quality areas)
3. Add manual track tweaking (drag control points to fix bad sections)
4. Worst case: pivot to curated tracks inspired by real locations

---

## Testing Strategy

### What Gets Strong TDD (Deterministic Systems)
These systems have clear right/wrong answers. Write full specs upfront.

- Protocol encoding/decoding
- OSM parsing (input → output)
- Road graph construction
- Track loop/sprint detection
- Checkpoint ordering
- Room state machine transitions
- Race logic (laps, placement, elimination)
- Serialization (MessagePack)
- Lat/lon → 3D coordinate projection
- API client caching logic
- Rate limiting

### What Gets Playtest Loops (Feel & Quality)
These are discovered through iteration, not specification. Use instrumentation and rapid feedback.

- Car handling / drift tuning
- Camera feel and responsiveness
- Track readability from top-down view
- Item fairness on generated tracks
- Visual clarity of road/building/nature
- Track quality scoring heuristics
- Performance tuning

### What Gets Resilience Tests (Edge Cases)
Added as a dedicated test bucket:

- [ ] Simulated high latency (200ms+): game remains playable
- [ ] Packet loss (5-10%): prediction hides gaps
- [ ] API timeout: graceful error, suggest retry
- [ ] Partial/malformed OSM data: builds what it can, doesn't crash
- [ ] Track generation timeout (>10s): abort and suggest different area
- [ ] Room owner disconnects: ownership transfers or room dissolves gracefully
- [ ] Reconnect during race: player rejoins with correct state
- [ ] Out-of-order snapshots: handled without visual glitch
- [ ] Browser suspend/resume (mobile tab switch): recovers
- [ ] Repeated race sessions in one tab: no memory leaks (soak test)
- [ ] Abuse: rapid room creation rate-limited
- [ ] Abuse: bbox validation rejects absurdly large/small areas

### Test Specs Per Milestone

**M1 - Scaffolding**:
- [ ] Client builds with `pnpm build`
- [ ] Server compiles with `cargo build`
- [ ] Docker Compose starts both services
- [ ] Client fetches data from server HTTP endpoint
- [ ] Pre-commit hooks run and pass

**M2 - Map Pipeline**:
- [ ] Overpass query returns valid JSON for a known bounding box
- [ ] OSM parser extracts roads with correct highway types from fixture data
- [ ] OSM parser extracts building footprints with estimated heights
- [ ] OSM parser extracts water/park/forest polygons
- [ ] Cache stores and retrieves map data by bbox hash
- [ ] Cache expires after configured TTL
- [ ] Projection: known lat/lon pairs convert to expected XZ within 1m tolerance
- [ ] Road widths match highway type lookup (fixture-based)
- [ ] Building count matches fixture (no silent data loss)
- [ ] `/api/map` endpoint returns parseable MapData
- [ ] API timeout returns error response, doesn't hang
- [ ] Invalid bbox returns 400 with helpful message
- [ ] Bbox size limit enforced (rejects >500m)

**M3 - Track Generation**:
- [ ] Road graph identifies intersections in test fixture
- [ ] Edges connect correct nodes with correct lengths
- [ ] Loop finder discovers known cycle in test graph
- [ ] Loops filtered by length (rejects <1000m and >3000m)
- [ ] Sprint route finder selects A→B path in tree graph
- [ ] Checkpoints spaced within 20% of target interval
- [ ] Start line placed on a road (not in a building)
- [ ] Track quality score rejects degenerate cases (straight line, tiny loop)
- [ ] Fallback to sprint when no loop found in sparse graph
- [ ] Track generation completes within 5 seconds for 500m bbox

**M4 - Core Engine + Physics**:
- [ ] Rapier WASM initializes without error
- [ ] Three.js renderer creates and renders a frame
- [ ] Camera follows target position (position within epsilon after N frames)
- [ ] Input: WASD maps to correct throttle/brake/steer values
- [ ] Fixed-timestep: physics tick count correct after simulated time
- [ ] Car accelerates forward when throttle applied (velocity increases)
- [ ] Car decelerates when brake applied (velocity decreases)
- [ ] Steering changes car heading (angular velocity nonzero)
- [ ] Drift: lateral velocity persists longer when braking + turning
- [ ] Car bounces off static cuboid (restitution test)
- [ ] Speed capped at maxSpeed (velocity never exceeds)
- [ ] Surface detection: raycast on road returns "road"
- [ ] Surface detection: raycast on grass returns "grass"
- [ ] Off-road: increased damping when on grass surface
- [ ] Resize: renderer and camera update on window resize
- [ ] Generated map loads and car can drive on roads
- [ ] No physics glitches at road intersections (car doesn't fall through)

**M5 - Single-Player Race**:
- [ ] Race state machine: transitions Waiting → Countdown → Racing → Finished
- [ ] Countdown: 3-2-1-GO timing correct
- [ ] Checkpoint: crossing in order advances counter
- [ ] Checkpoint: crossing out of order is ignored
- [ ] Lap: completing all checkpoints increments lap count
- [ ] Finish: race ends when lap target reached
- [ ] Sprint: race ends when finish line reached
- [ ] Race timer starts at GO, stops at finish
- [ ] Results: finish time recorded correctly

**M6 - Multiplayer**:
- [ ] Server starts and accepts WebSocket connections
- [ ] CreateRoom returns unique room_id
- [ ] JoinRoom adds player (room state includes them)
- [ ] Max 8 players per room enforced (9th rejected)
- [ ] Input messages reach server and are buffered
- [ ] Server tick loop: processes inputs + steps physics at 60Hz
- [ ] Snapshot broadcast: all players receive world state
- [ ] Client prediction: local car moves on input (no visible delay)
- [ ] Reconciliation: state corrects when prediction diverges
- [ ] Correction frequency under 5% of snapshots in normal conditions
- [ ] Correction magnitude under 0.5m in normal conditions
- [ ] Entity interpolation: remote cars move smoothly (no teleporting)
- [ ] Room auto-cleanup after 5 minutes empty
- [ ] Disconnect: player removed from room
- [ ] Join mid-race: receives current state and can spectate

**M7 - Game Modes**:
- [ ] Circuit: lap counting works, race ends after N laps (configurable)
- [ ] Sprint: race ends when first player reaches B
- [ ] Elimination: last-place eliminated every 30s, spectator mode works
- [ ] Time Trial: solo, completes without errors
- [ ] Mode selection communicated to all players before race
- [ ] Elimination removes car from physics world

**M8 - Power-ups**:
- [ ] Item boxes spawn at track-designated points
- [ ] Pickup assigns random item (weighted by position)
- [ ] SpeedBoost: speed increases for duration, then returns to normal
- [ ] Missile: travels forward, despawns after lifetime or hit
- [ ] OilSlick: placed on road, reduces grip on contact
- [ ] Shield: absorbs next hit
- [ ] Max 1 item per car (pickup ignored when holding)
- [ ] Items respawn after cooldown
- [ ] All item state server-authoritative (client shows optimistic)
- [ ] Items suppressed on very narrow road segments (<4m width)

**M9 - UI**:
- [ ] Search input returns geocoding results
- [ ] Leaflet map renders with tiles
- [ ] Bounding box selection limited to 500m
- [ ] Lobby shows connected players
- [ ] Ready toggle visible to all players
- [ ] HUD displays speed, position, lap, item
- [ ] Countdown sequence plays correctly
- [ ] Results screen shows standings
- [ ] E2E: full flow map selection → race → results completes

**M10 - Discord**:
- [ ] Discord environment detected in iframe
- [ ] OAuth2 token exchange succeeds
- [ ] Participant list tracks voice channel changes
- [ ] Standalone mode unaffected by Discord code
- [ ] URL mappings route through Discord proxy

**M11 - Polish**:
- [ ] Audio: engine pitch scales with speed
- [ ] Collision SFX on contact events
- [ ] Drift particles during drift state
- [ ] Scene loads within memory budget (define per platform)
- [ ] No detectable memory growth over 10 consecutive races (soak)
- [ ] Quality auto-scales based on measured frame time

---

## Room State Machine

```
Idle → Lobby → LoadingMap → Countdown → Racing → Results → Lobby (rematch)
                                                          → Dissolving
```

**Per-state rules:**

| State | Who Can Join | Disconnect Behavior | Owner Leave |
|-------|-------------|---------------------|-------------|
| Lobby | Anyone with room code | Removed from player list | Ownership transfers to next player |
| LoadingMap | No new joins | Kicked, rejoin as observer | Wait for load to finish |
| Countdown | No new joins | Removed, race continues | Race continues |
| Racing | Join as spectator only | Car removed, marked DNF | Race continues |
| Results | Anyone with room code | No effect | No effect |
| Dissolving | No | N/A | N/A |

**Reconnect**: During Racing, a disconnected player has 30 seconds to reconnect. Their car continues with last known input. If they reconnect, they resume control. If not, car is removed (DNF).

---

## Track Quality System

### Heuristics for Scoring Generated Tracks

```
score = (turn_density * 0.3)
      + (road_variety * 0.2)
      + (width_score * 0.2)
      + (recovery_space * 0.15)
      + (distinctiveness * 0.15)
```

| Factor | Good | Bad |
|--------|------|-----|
| Turn density | 1 turn per 150m | <1 per 500m (boring straight) |
| Road variety | Mix of widths | All same type |
| Width | Average >6m | <4m (too narrow for racing) |
| Recovery space | Open areas beside road | Buildings tight on both sides |
| Distinctiveness | Varied scenery, landmarks | Monotonous residential grid |

**Thresholds**:
- Score > 0.7: "Great track" - use as-is
- Score 0.4-0.7: "Okay track" - usable with warning
- Score < 0.4: **Reject** - suggest nearby alternatives

### Track Cleanup Pipeline (gameplay over cartographic purity)
1. Widen roads below 5m to minimum 5m (playable width)
2. Simplify complex intersections (>4 roads meeting) - merge into single junction zone
3. Remove non-essential dead-end branches from track area
4. Suppress building geometry that occludes >30% of track visibility
5. Flatten problematic terrain spikes within 10m of road surface

---

## Fallback & UX for Weak Locations

**When generation fails or produces weak tracks:**
1. "This area doesn't have great racing roads. Try these nearby spots:" + 3 suggestions
2. Featured preset locations (known-good, pre-cached for instant play):
   - Urban grid: Manhattan, central Barcelona, Tokyo Shibuya
   - Winding: San Francisco hills, Monaco, Amalfi Coast
   - Suburban: typical US suburb, UK village
   - Mix: Amsterdam canals, Paris arrondissements
3. "Random good location" button (draws from curated list)
4. Recently played locations (personal + popular)

**Generation progress UX:**
1. "Fetching road data..." (Overpass)
2. "Building 3D world..." (geometry generation)
3. "Finding best route..." (track generation)
4. Progress bar with each stage, not a spinner

---

## Physics Authority Model

**Primary model**: Server-authoritative for all game state.

**Fallback plan if client/server physics diverge too often:**

| Concern | Authority | Fallback if Divergence > 5% |
|---------|-----------|---------------------------|
| Local car position | Client-predicted, server-reconciled | Increase reconciliation smoothing window |
| Remote car positions | Server-authoritative, client-interpolated | Already smooth by design |
| Collisions (car-car) | Server-authoritative | Cosmetic-only local collision response |
| Collisions (car-building) | Server-authoritative | Client treats as cosmetic bumps, server corrects |
| Items/weapons | Server-authoritative | No change (already server-only) |
| Race state | Server-authoritative | No change |

**Measurement**: Log correction frequency and magnitude. Dashboard in debug mode.
- Acceptable: <5% of snapshots require correction, <0.5m average correction
- Warning: 5-15% corrections or >0.5m average
- Critical: >15% corrections or >2m average (fall back to pure server authority + smoothing)

---

## API Shielding & Rate Limiting

| API | Rate Limit | Cache TTL | Abuse Protection |
|-----|------------|-----------|------------------|
| Overpass | 1 req/2 sec per IP | 1 week | Bbox size limit (500m), per-session cooldown (5 sec between requests) |
| Nominatim | 1 req/sec | 24 hours | Debounced search, per-session limit (30 searches/min) |
| Open Elevation | 10 req/sec | 1 week | Batched (200 points/req), tied to map cache |

**Server-side protections:**
- Bbox validation: reject if side < 100m or > 500m
- Per-IP rate limit: 10 map generations per hour
- Per-session rate limit: 3 concurrent pending requests
- Cache-aware: serve from cache before hitting external APIs
- Abuse monitoring: log unusual patterns

---

## Elevation Strategy (Deferred for MVP)

**MVP (M1-M8)**: Flat terrain. All roads at Y=0. Buildings extruded from flat ground. No elevation API calls.

**Post-MVP enhancement**:
- Simplified elevation: broad hills only, no harsh slopes
- Normalize terrain within 10m of roads to prevent physics issues
- Visual-only elevation for areas far from track
- Full elevation interaction deferred until proven needed

---

## Architecture Overview

**Monorepo** with `client/` (TypeScript/Three.js) and `server/` (Rust workspace).

**Rapier physics** runs on both server (native Rust) and client (WASM) with `enhanced-determinism`. If divergence exceeds thresholds, fallback to pure server authority with cosmetic smoothing.

The Rust server acts as **proxy/cache** for all external APIs (Overpass, Nominatim), solving Discord CSP restrictions and enabling caching. Standalone web is the primary platform; Discord Activity is layered on top later.

**Protocol**: MessagePack over WebSocket. Physics at 60Hz, snapshots broadcast at 20Hz.

## Project Structure

```
real-micro-machines/
├── client/                          # Vite + TypeScript + Three.js
│   ├── src/
│   │   ├── main.ts                  # Entry point
│   │   ├── core/                    # Game loop, renderer, camera, input, clock
│   │   ├── physics/                 # Rapier WASM, car body, track colliders, prediction
│   │   ├── vehicles/                # Car visual + physics, config, effects
│   │   ├── map/                     # OSM parser, projection, terrain/road/building/nature builders
│   │   ├── gameplay/                # Race, game modes, checkpoints, power-ups, weapons
│   │   ├── network/                 # WebSocket, protocol, lobby, state sync
│   │   ├── ui/                      # Screens (menu, map selector, lobby, HUD, results)
│   │   ├── discord.ts               # Discord SDK (Phase C only)
│   │   └── utils/                   # Asset loader, object pool, debug
│   ├── e2e/                         # Playwright E2E tests
│   ├── vitest.config.ts
│   └── playwright.config.ts
│
├── server/                          # Rust workspace
│   └── crates/
│       ├── rmm-server/              # Axum HTTP + WS server
│       ├── rmm-game/                # Room, game loop, physics, race, modes, items
│       ├── rmm-network/             # WS handler, protocol, sessions, snapshots
│       ├── rmm-map/                 # Overpass, nominatim, elevation, track gen, cache
│       └── rmm-auth/                # Discord OAuth2, standalone sessions
│
├── shared/                          # Protocol documentation + schemas
├── docker-compose.yml
├── lefthook.yml
├── .env.example
└── package.json                     # pnpm workspace root
```

## Game Modes

| Mode | Track Type | End Condition | Players |
|------|-----------|---------------|---------|
| Circuit | Loop, 1-10 laps (default 3) | All finish or timeout | 1-8 |
| Sprint | A→B point-to-point | First to B, or all finish | 1-8 |
| Elimination | Loop, continuous | Last player standing | 2-8 |
| Time Trial | Any, solo | Complete or quit | 1 |

## Vehicles Architecture

MVP: 1 car model with 8 color variants. Stats from `CarConfig`:

```typescript
interface CarConfig {
  acceleration: number;    // 15-25
  brakeForce: number;      // 20-30
  maxSpeed: number;        // 20-30 m/s
  steerStrength: number;   // 3-6
  normalGrip: number;      // 0.8-0.95
  driftGrip: number;       // 0.2-0.4
  mass: number;            // 0.8-1.5
  bounciness: number;      // 0.3-0.5
}
```

Data-driven, loaded from config file. Architecture supports 6+ car types post-MVP.

## Key Dependencies

**Client**: `three`, `@dimforge/rapier3d-compat`, `msgpackr`, `leaflet`, `vite`, `vite-plugin-wasm`, `vitest`, `@playwright/test`, `typescript`, `eslint`

**Server**: `axum`, `tokio`, `tokio-tungstenite`, `rapier3d` (enhanced-determinism), `rmp-serde`, `reqwest`, `serde`, `uuid`, `tracing`, `dotenv`

**Discord (Phase C only)**: `@discord/embedded-app-sdk`

**Dev tools**: `pnpm`, `lefthook`, `cargo-watch`, `docker`, `docker-compose`

## Key Risks (with Mitigations)

| Risk | Severity | Mitigation |
|------|----------|------------|
| Generated tracks not fun enough | **Critical** | M4 validation gate. Track quality scoring. Reject bad tracks. Featured locations. |
| OSM data quality varies | High | Sensible defaults. Track cleanup pipeline. Graceful failure UX. |
| Client/server physics diverge | High | Measurement thresholds. Fallback to pure server authority + smoothing. |
| Multiplayer complexity | Medium | Prove single-player first. Add multiplayer only after core loop works. |
| Discord + standalone = two products | Medium | Standalone first. Discord layered on later. Shared code, not dual architecture. |
| Pre-race friction (too many clicks) | Medium | Featured locations. "Random good spot" button. Async generation preview. |
| API rate limits / abuse | Medium | Server-side caching, rate limits, bbox validation. |
| Mobile performance | Medium | Quality auto-scaling. Test early on real devices. |

## Verification Plan

| Gate | What Must Be True |
|------|-------------------|
| M1 | Docker Compose up, services communicate |
| M2 | Real location renders as 3D roads + buildings |
| M3 | >60% of 50 test locations produce raceable route |
| **M4** | **"Is it fun?" - driving on generated maps feels good (playtest)** |
| M5 | Complete solo race start to finish |
| M6 | Two tabs racing, smooth multiplayer |
| M7 | All 4 modes work |
| M8 | Items feel fair on generated tracks |
| M9 | Full UI flow, <5 clicks to first race |
| M10 | Works as Discord Activity |
| M11 | 60fps benchmark, no memory leaks over 10 races |
| M12 | All KPIs green, ready to ship |

## Build Order

```
M1 → M2 → M3 → M4 (CONCEPT VALIDATION GATE)
  → M5 → M6 (multiplayer)
  → M7 → M8 (modes + items)
  → M9 → M10 (UI + Discord)
  → M11 → M12 (polish + release)
```

**If M4 fails**: iterate on track quality, road distortion, or location curation before proceeding.

---

# Detailed Milestone Breakdowns

Each milestone is broken into **stories** (units of work). Each story has acceptance criteria, estimated complexity, and key files. Stories within a milestone can often be worked in parallel.

---

## M1: Project Scaffolding (v0.1.0)

**Goal**: Monorepo builds, Docker runs both services, basic client-server communication works.

### Stories

#### M1-S1: Initialize monorepo and client project
**Complexity**: Small
**Files**: `package.json`, `pnpm-workspace.yaml`, `client/package.json`, `client/tsconfig.json`, `client/vite.config.ts`, `client/index.html`, `client/src/main.ts`
**Acceptance**:
- pnpm workspace configured with `client/` package
- Vite project with TypeScript strict mode
- `vite-plugin-wasm` and `vite-plugin-top-level-await` configured
- `pnpm dev` serves a blank page at localhost:5173
- `pnpm build` produces `client/dist/` with no errors

#### M1-S2: Initialize Rust server workspace
**Complexity**: Small
**Files**: `server/Cargo.toml`, `server/crates/rmm-server/Cargo.toml`, `server/crates/rmm-server/src/main.rs`, `server/crates/rmm-server/src/config.rs`
**Acceptance**:
- Cargo workspace with `rmm-server` crate
- Axum server starts on configurable port
- Health check endpoint (`GET /health`) returns 200
- `cargo build` succeeds with no warnings
- `cargo test` passes (even if no tests yet)

#### M1-S3: Basic HTTP communication
**Complexity**: Small
**Files**: `client/src/main.ts`, `server/crates/rmm-server/src/main.rs`
**Acceptance**:
- Server has `GET /api/ping` endpoint returning `{"status": "ok"}`
- Client fetches `/api/ping` on load and logs result
- CORS configured for local development

#### M1-S4: Docker Compose setup
**Complexity**: Medium
**Files**: `docker-compose.yml`, `Dockerfile.client`, `server/Dockerfile`, `.env.example`
**Acceptance**:
- `docker-compose up` starts both client and server
- Client accessible at localhost:5173
- Server accessible at localhost:3001
- Hot reload works for client source changes
- Server rebuilds on Rust source changes (cargo-watch)
- `.env.example` documents all required env vars

#### M1-S5: Test infrastructure
**Complexity**: Small
**Files**: `client/vitest.config.ts`, `client/src/main.test.ts`, `client/playwright.config.ts`
**Acceptance**:
- `pnpm test` runs Vitest, at least 1 passing test
- `pnpm test:e2e` runs Playwright (placeholder test)
- `cargo test` in server workspace passes
- Coverage reporting configured

#### M1-S6: Git setup and pre-commit hooks
**Complexity**: Small
**Files**: `lefthook.yml`, `.gitignore`
**Acceptance**:
- Git initialized with `develop` and `main` branches
- GitHub repo created
- lefthook runs on pre-commit: lint, typecheck, test (changed files)
- `.gitignore` covers node_modules, target, dist, .env, cache files

#### M1-S7: GitHub Projects board
**Complexity**: Small
**Acceptance**:
- GitHub Project created with columns: Backlog, In Progress, Review, Done
- M1 milestone created with all M1 stories as issues
- Labels created: `milestone/M1` through `milestone/M12`, `story`, `bug`, `enhancement`

**M1 Definition of Done**: `docker-compose up` starts both services, client fetches from server, all tests pass, hooks run.

---

## M2: Map Pipeline (v0.2.0)

**Goal**: Fetch real OSM data for any location, parse it, generate 3D geometry, render it.

### Stories

#### M2-S1: Overpass API client
**Complexity**: Medium
**Files**: `server/crates/rmm-map/src/lib.rs`, `server/crates/rmm-map/src/overpass.rs`, `server/crates/rmm-map/src/osm_types.rs`
**Depends on**: M1-S2
**Acceptance**:
- Takes bounding box (south, west, north, east), returns raw Overpass JSON
- Query fetches: roads (`highway`), buildings (`building`), water (`natural=water`), parks (`leisure=park`), forests (`landuse=forest`)
- Uses `out body geom;` for inline coordinates
- Timeout handling: returns error after 30 seconds
- Rate limiting: max 1 request per 2 seconds
- Test: known bbox returns expected road count (fixture-based)

#### M2-S2: OSM data parser
**Complexity**: Medium
**Files**: `server/crates/rmm-map/src/osm_types.rs`
**Depends on**: M2-S1
**Acceptance**:
- Parses Overpass JSON into typed `MapData` struct
- Extracts roads with: id, highway type, points (lat/lon), estimated width, name, oneway, lanes
- Extracts buildings with: id, footprint polygon, estimated height (from `height` tag, `building:levels * 3m`, or default 8m)
- Extracts water/park/forest polygons
- Width lookup table: motorway=14m, trunk=12m, primary=10m, secondary=8m, tertiary=7m, residential=6m, service=4m, footway=2m
- Test: fixture JSON produces expected MapData fields

#### M2-S3: Server-side map cache
**Complexity**: Small
**Files**: `server/crates/rmm-map/src/cache.rs`
**Depends on**: M2-S1
**Acceptance**:
- Disk-based cache using bbox hash as key
- Stores processed MapData as MessagePack
- TTL: configurable, default 1 week
- Cache hit: returns data without API call
- Cache miss: fetches from API, stores, returns
- Test: second request for same bbox returns cached data

#### M2-S4: Map API endpoint
**Complexity**: Small
**Files**: `server/crates/rmm-server/src/main.rs`
**Depends on**: M2-S1, M2-S2, M2-S3
**Acceptance**:
- `GET /api/map?south=X&west=X&north=X&east=X` returns MapData as MessagePack
- Bbox validation: rejects if side < 100m or > 500m (returns 400)
- Invalid params return 400 with message
- Server error returns 500 with safe error message
- Test: endpoint returns parseable MapData for known bbox

#### M2-S5: Lat/lon to 3D coordinate projection
**Complexity**: Small
**Files**: `client/src/map/Projection.ts`, `client/src/map/Projection.test.ts`
**Acceptance**:
- Local tangent plane projection centered on bbox center
- Converts lat/lon to XZ meters (Y is up in Three.js)
- Accounts for longitude scaling by latitude (cos correction)
- Test: known lat/lon pairs produce expected XZ within 1m tolerance
- Test: center of bbox maps to (0, 0)

#### M2-S6: Road geometry builder
**Complexity**: Large
**Files**: `client/src/map/RoadBuilder.ts`, `client/src/map/RoadBuilder.test.ts`
**Depends on**: M2-S5
**Acceptance**:
- Takes road polyline + width, produces Three.js BufferGeometry
- Triangle strip along road centerline with perpendicular offsets
- Smooth corners using averaged direction vectors at junctions
- Roads rendered at Y=0.05 (above ground to prevent z-fighting)
- Different road types get different material colors/widths
- Test: straight road produces expected vertex count
- Test: curved road vertices are within width tolerance of centerline

#### M2-S7: Building geometry builder
**Complexity**: Medium
**Files**: `client/src/map/BuildingBuilder.ts`, `client/src/map/BuildingBuilder.test.ts`
**Depends on**: M2-S5
**Acceptance**:
- Takes building footprint polygon + height, produces ExtrudeGeometry
- All buildings batch-merged into single BufferGeometry (draw call optimization)
- Building color varies by type (warm=residential, gray=commercial, dark=industrial)
- Test: building count in merged geometry matches input count
- Test: total draw calls for buildings is 1 (or small constant)

#### M2-S8: Nature and water builder
**Complexity**: Medium
**Files**: `client/src/map/NatureBuilder.ts`
**Depends on**: M2-S5
**Acceptance**:
- Trees: InstancedMesh low-poly cones/spheres placed within park/forest polygons
- Poisson disk sampling for natural spacing
- Water: flat blue semi-transparent plane at Y=-0.1
- Parks: green-tinted ground plane
- Test: trees placed only within polygon bounds

#### M2-S9: Map loader and scene integration
**Complexity**: Medium
**Files**: `client/src/map/MapLoader.ts`, `client/src/main.ts`
**Depends on**: M2-S4, M2-S6, M2-S7, M2-S8
**Acceptance**:
- Fetches MapData from server, feeds to builders, adds meshes to Three.js scene
- Loading progress reported (for future progress UI)
- Basic Three.js scene with camera looking down at the map
- Mouse/keyboard orbit controls for inspecting the generated world
- **Can see a real location rendered in 3D in the browser**

#### M2-S10: Ground plane and terrain
**Complexity**: Small
**Files**: `client/src/map/TerrainBuilder.ts`
**Acceptance**:
- Flat green ground plane covering the bbox area
- Extends slightly beyond bbox to avoid visible edges
- Roads and buildings sit on top of it
- (Elevation deferred to post-MVP)

**M2 Definition of Done**: Pick a real-world location (e.g., central London), see 3D roads, buildings, trees, water rendered in the browser. Can orbit camera to inspect.

---

## M3: Track Generation (v0.3.0)

**Goal**: Convert road network into raceable circuits and sprint routes with checkpoints.

### Stories

#### M3-S1: Road graph construction
**Complexity**: Medium
**Files**: `server/crates/rmm-map/src/track_gen.rs`
**Depends on**: M2-S2
**Acceptance**:
- Converts parsed roads into graph: nodes (intersections) + edges (road segments)
- Two roads sharing an endpoint node = intersection
- Edge stores: length in meters, road type, intermediate geometry points
- Test: known road layout produces expected node/edge count
- Test: T-junction creates correct 3-edge node

#### M3-S2: Circuit loop finder
**Complexity**: Large
**Files**: `server/crates/rmm-map/src/track_gen.rs`
**Depends on**: M3-S1
**Acceptance**:
- Finds simple cycles in road graph
- Filters by total length: 1000m - 3000m
- Uses DFS with cycle detection (Johnson's algorithm or similar)
- Limits search depth to prevent exponential blowup (max 20 edges per cycle)
- Returns top N candidate loops ranked by score
- Test: finds known cycle in test fixture graph
- Test: rejects cycles shorter than 1000m
- Test: completes within 5 seconds for 500m bbox-sized graph

#### M3-S3: Track quality scoring
**Complexity**: Medium
**Files**: `server/crates/rmm-map/src/track_gen.rs`
**Depends on**: M3-S2
**Acceptance**:
- Scores each candidate loop: turn density (0.3), road variety (0.2), width (0.2), recovery space (0.15), distinctiveness (0.15)
- Score > 0.7: great, 0.4-0.7: usable, < 0.4: reject
- Test: straight-line loop scores low
- Test: winding mixed-road loop scores high
- Test: very narrow roads reduce score

#### M3-S4: Sprint route finder
**Complexity**: Medium
**Files**: `server/crates/rmm-map/src/track_gen.rs`
**Depends on**: M3-S1
**Acceptance**:
- Finds longest interesting A→B path through road graph
- Path must be connected and non-repeating
- Scored by same quality heuristics as circuits
- Used as fallback when no loops found
- Test: finds A→B path in tree graph (no loops possible)
- Test: path length within expected range

#### M3-S5: Checkpoint and start line placement
**Complexity**: Medium
**Files**: `server/crates/rmm-map/src/track_gen.rs`
**Depends on**: M3-S2, M3-S4
**Acceptance**:
- Checkpoints placed every ~200m along track (within 20% tolerance)
- Start/finish line on widest road segment in the loop
- Power-up spawn points at regular intervals between checkpoints
- Direction of travel assigned (CW or CCW based on turn analysis)
- Test: checkpoints evenly distributed along test track
- Test: start line on a road, not inside a building

#### M3-S6: Track generation API endpoint
**Complexity**: Small
**Files**: `server/crates/rmm-server/src/main.rs`
**Depends on**: M3-S2, M3-S4, M3-S5
**Acceptance**:
- `GET /api/track?south=X&west=X&north=X&east=X` returns GeneratedTrack
- If no good track found, returns alternatives or error with suggestion
- Reuses cached MapData from M2
- Test: endpoint returns valid track for known good location
- Test: endpoint returns helpful error for sparse location

#### M3-S7: Track quality validation suite
**Complexity**: Medium
**Files**: `server/tests/track_quality.rs`
**Depends on**: M3-S6
**Acceptance**:
- Test 50 diverse locations (urban grids, winding roads, suburban, rural, sparse)
- Record: success rate, quality scores, generation time
- **>60% must produce a raceable route (score > 0.4)**
- Document which location types work well and which don't
- This is the first concept validation data point

**M3 Definition of Done**: >60% of 50 test locations produce a quality-scored raceable track. Track endpoints return data within 5 seconds.

---

## M4: Core Engine + Physics (v0.4.0) - CONCEPT VALIDATION GATE

**Goal**: Drive a car on generated maps. Prove the core fantasy is fun.

### Stories

#### M4-S1: Three.js renderer and scene setup
**Complexity**: Medium
**Files**: `client/src/core/Renderer.ts`, `client/src/core/Game.ts`
**Acceptance**:
- WebGLRenderer with antialias, correct pixel ratio, tone mapping
- Resizes on window resize (ResizeObserver)
- Basic lighting: ambient + directional (simulating sun)
- Ground shadow (simple blob, not shadow maps for MVP)
- Test: renderer creates canvas and renders a frame
- Test: resize updates viewport correctly

#### M4-S2: Top-down camera system
**Complexity**: Medium
**Files**: `client/src/core/Camera.ts`
**Acceptance**:
- Perspective camera at ~75-80 degree angle (near top-down, slight tilt)
- Follows target position with smooth lerp damping
- Shows ~200-300m of road ahead of car
- Rotates to match car heading (car always faces "up" on screen)
- Smooth transition, no jitter
- *Feel-based tuning*: smooth enough to anticipate turns 2+ seconds ahead

#### M4-S3: Input system
**Complexity**: Small
**Files**: `client/src/core/Input.ts`, `client/src/core/Input.test.ts`
**Acceptance**:
- Keyboard: WASD/arrows → `{ throttle: 0-1, brake: 0-1, steer: -1 to 1, useItem: bool }`
- Gamepad: analog sticks + triggers → same InputState
- Polling-based (read state each frame, not event-driven)
- Test: W key → throttle=1, A key → steer=-1, etc.
- Test: no input → all zeros

#### M4-S4: Fixed-timestep game loop
**Complexity**: Small
**Files**: `client/src/core/Game.ts`, `client/src/core/Clock.ts`
**Acceptance**:
- Accumulator-based fixed timestep at 60Hz (FIXED_DT = 1/60)
- Multiple fixedUpdate steps per frame if needed
- Cap max accumulated time to prevent spiral of death
- Render interpolation between last two physics states
- Test: after simulating 1 second, fixedUpdate called ~60 times

#### M4-S5: Rapier WASM physics world
**Complexity**: Medium
**Files**: `client/src/physics/PhysicsWorld.ts`
**Acceptance**:
- Rapier WASM loads via async init
- World created with gravity (0, -9.81, 0)
- Step function callable from game loop
- Add/remove rigid bodies and colliders
- Test: WASM initializes without error
- Test: body with gravity falls over time

#### M4-S6: Car physics body
**Complexity**: Large (feel-intensive, lots of tuning)
**Files**: `client/src/physics/CarBody.ts`, `client/src/vehicles/CarConfig.ts`
**Acceptance**:
- Dynamic RigidBody with Cuboid collider (car-sized)
- Force-based arcade model: forward impulse (throttle), torque (steering), lateral friction cancellation (grip)
- Drift mechanic: braking while turning reduces lateral friction
- Linear damping for natural deceleration
- Angular damping to prevent endless spinning
- Speed cap at maxSpeed
- CarConfig data-driven tuning
- Test: throttle increases forward velocity
- Test: brake decreases forward velocity
- Test: steer creates angular velocity
- Test: drift mode has lower lateral friction than normal mode
- Test: speed never exceeds maxSpeed
- *Playtest*: "Does this feel fun to drive?"

#### M4-S7: Car visual model
**Complexity**: Small
**Files**: `client/src/vehicles/Car.ts`
**Acceptance**:
- Placeholder car model (colored box or simple low-poly shape)
- Position and rotation synced from physics body each frame
- 8 color variants (for future multiplayer)
- Visible from top-down camera

#### M4-S8: Track colliders from map data
**Complexity**: Medium
**Files**: `client/src/physics/TrackColliders.ts`
**Depends on**: M2-S7, M4-S5
**Acceptance**:
- Buildings become static Cuboid colliders matching footprint and height
- Ground plane becomes static plane collider
- Colliders tagged with surface type (road, grass, water, building)
- Car collides with buildings (bounces off)
- Car drives on road surface
- Test: car doesn't fall through ground
- Test: car bounces off building

#### M4-S9: Surface detection and off-road penalty
**Complexity**: Medium
**Files**: `client/src/vehicles/Car.ts`, `client/src/physics/CarBody.ts`
**Depends on**: M4-S8
**Acceptance**:
- Raycast downward from car detects surface type
- Road: full speed, normal physics
- Grass: increased linear damping (natural slowdown to ~60%)
- Water: trigger respawn (teleport to last road position)
- Building: collision handled by physics (bounce)
- *Playtest*: Off-road penalty feels punishing but fair

#### M4-S10: Integration - drive on generated map
**Complexity**: Medium
**Depends on**: M2-S9, M3-S6, M4-S1 through M4-S9
**Acceptance**:
- Load a real location's map data and track
- Generate 3D world with buildings, roads, nature
- Create physics colliders from map geometry
- Spawn car at track start line
- Drive freely on the generated roads
- Camera follows car
- Buildings are solid, grass is slow, water respawns
- **PLAYTEST SESSION**: Drive 5 different generated tracks, record:
  - Is it fun? (1-5 rating per track)
  - Can you read the track from the camera angle?
  - Do physics glitches occur at intersections?
  - Does the car stay on generated roads at reasonable speeds?
  - Does off-road feel punishing but fair?

**M4 Definition of Done**: Playtest verdict is positive. Driving on generated maps feels good enough to invest in multiplayer.

**IF GATE FAILS**: Stop. Iterate on:
1. Track distortion (widen roads, simplify intersections)
2. Camera angle/zoom adjustments
3. Physics tuning on real geometry
4. Location filtering (only allow high-quality areas)
5. Manual track editing tools
Do not proceed to M5 until driving feels fun.

---

## M5: Single-Player Race (v0.5.0)

**Goal**: Complete a solo race from countdown to finish line.

### Stories

#### M5-S1: Race state machine
**Complexity**: Medium
**Files**: `client/src/gameplay/Race.ts`, `client/src/gameplay/Race.test.ts`
**Acceptance**:
- States: Waiting → Countdown → Racing → Finished
- Transitions triggered by events (all ready, countdown done, laps complete)
- Timer starts at Racing, stops at Finished
- Test: state transitions correctly in sequence
- Test: timer accumulates during Racing state only

#### M5-S2: Checkpoint system
**Complexity**: Medium
**Files**: `client/src/gameplay/Checkpoint.ts`
**Depends on**: M3-S5
**Acceptance**:
- Rapier sensor colliders at each checkpoint position
- Crossing checkpoint in order advances counter
- Crossing out of order is ignored (prevents shortcuts)
- All checkpoints hit = lap complete
- Visual: translucent gates over road at checkpoint positions
- Test: driving through checkpoints in order counts correctly
- Test: skipping a checkpoint doesn't advance

#### M5-S3: Lap counting and race finish
**Complexity**: Small
**Files**: `client/src/gameplay/Race.ts`
**Depends on**: M5-S2
**Acceptance**:
- Configurable lap count (default 3)
- Lap increments when all checkpoints completed
- Race finishes when target laps reached
- Finish time recorded
- Sprint mode: race ends when crossing B checkpoint
- Test: 3-lap race finishes after 3 complete checkpoint cycles

#### M5-S4: Countdown overlay
**Complexity**: Small
**Files**: `client/src/ui/hud/Countdown.ts`
**Acceptance**:
- 3-2-1-GO sequence displayed as large text overlay
- Each number displays for 1 second
- Car input disabled during countdown, enabled at GO
- Test: countdown timing correct (3 seconds total)

#### M5-S5: Basic HUD
**Complexity**: Small
**Files**: `client/src/ui/hud/HUD.ts`
**Acceptance**:
- Speed display (current car speed in km/h or m/s)
- Lap counter ("Lap 2/3")
- Race timer (mm:ss.ms)
- All rendered as HTML overlays, crisp text
- Updates every frame

#### M5-S6: Race completion and restart
**Complexity**: Small
**Files**: `client/src/ui/screens/Results.ts`
**Acceptance**:
- Results screen shows finish time, best lap time
- "Race Again" button (same track)
- "New Location" button (back to location picker - placeholder for now)
- Test: full race from countdown to results screen completes

**M5 Definition of Done**: Complete a solo 3-lap circuit race on a generated map, see countdown, HUD, and results screen.

---

## M6: Multiplayer (v0.6.0)

**Goal**: Two or more players racing each other in real-time.

### Stories

#### M6-S1: WebSocket server infrastructure
**Complexity**: Medium
**Files**: `server/crates/rmm-network/src/lib.rs`, `server/crates/rmm-network/src/ws_handler.rs`, `server/crates/rmm-network/src/session.rs`
**Acceptance**:
- Axum WebSocket upgrade handler
- Per-connection Session with unique player ID
- Message routing between WebSocket and game logic
- Graceful disconnect handling
- Test: client connects, server assigns ID, client disconnects cleanly

#### M6-S2: Protocol definition
**Complexity**: Medium
**Files**: `server/crates/rmm-network/src/protocol.rs`, `client/src/network/Protocol.ts`, `shared/PROTOCOL.md`
**Acceptance**:
- MessagePack serialization for all game messages
- Client→Server: CreateRoom, JoinRoom, SetReady, PlayerInput, ChatMessage
- Server→Client: RoomCreated, RoomState, PlayerJoined, PlayerLeft, GameStart, WorldSnapshot, RaceEvent, GameEnd, AuthResult
- Documented in PROTOCOL.md
- Test: encode/decode roundtrip for every message type (both Rust and TypeScript)

#### M6-S3: Room management
**Complexity**: Large
**Files**: `server/crates/rmm-game/src/room.rs`, `server/crates/rmm-game/src/player.rs`
**Acceptance**:
- Create room (generates 6-char code)
- Join room by code (max 8 players)
- Room state machine: Idle → Lobby → LoadingMap → Countdown → Racing → Results → Dissolving
- Player ready toggle
- Owner can start when all ready
- Auto-cleanup after 5 minutes empty
- Ownership transfers on owner disconnect
- Test: create room, join, ready, start sequence
- Test: 9th player rejected
- Test: owner disconnect transfers ownership
- Test: empty room auto-deletes

#### M6-S4: Server game tick loop
**Complexity**: Large
**Files**: `server/crates/rmm-game/src/game_loop.rs`, `server/crates/rmm-game/src/car_physics.rs`
**Depends on**: M6-S3
**Acceptance**:
- Tokio task per room, ticking at 60Hz via `tokio::time::interval`
- Processes buffered player inputs (2-3 frame buffer for jitter)
- Steps Rapier physics with same parameters as client
- Runs race logic (checkpoints, laps)
- Missing input: replay last known input
- Test: tick loop runs at correct rate
- Test: input buffer handles jitter correctly

#### M6-S5: Snapshot broadcasting
**Complexity**: Medium
**Files**: `server/crates/rmm-network/src/snapshot.rs`
**Depends on**: M6-S4
**Acceptance**:
- WorldSnapshot sent every 3 ticks (20Hz) to all room players
- Contains: tick number, per-player last processed input seq, car states (position, rotation, velocity), race state
- Serialized as MessagePack
- Test: snapshot contains correct data for all players in room
- Test: bandwidth per room < 20KB/s for 8 players

#### M6-S6: Client WebSocket connection
**Complexity**: Medium
**Files**: `client/src/network/Connection.ts`, `client/src/network/Lobby.ts`
**Acceptance**:
- WebSocket connect to server
- Auto-reconnect with exponential backoff (1s, 2s, 4s, 8s, max 30s)
- Create/join room
- Player list updates on join/leave
- Test: connects and receives room state
- Test: reconnects after disconnect

#### M6-S7: Client-side prediction
**Complexity**: Large
**Files**: `client/src/physics/Prediction.ts`
**Depends on**: M6-S5, M6-S6
**Acceptance**:
- Local car moves immediately on input (no delay)
- Input history recorded with sequence numbers
- On server snapshot: compare predicted state with server state
- If diverged: reset to server state, replay unprocessed inputs
- Correction smoothing (don't teleport, lerp toward correct position)
- Test: prediction matches server in zero-latency scenario
- Test: correction happens when state diverges

#### M6-S8: Entity interpolation
**Complexity**: Medium
**Files**: `client/src/network/StateSync.ts`
**Depends on**: M6-S5
**Acceptance**:
- Buffer last 3-5 snapshots
- Remote players rendered at ~100ms in the past (interpolation delay)
- Smooth interpolation between snapshot positions
- No teleporting or jittering
- Test: remote car moves smoothly between known snapshot positions

#### M6-S9: Divergence monitoring
**Complexity**: Small
**Files**: `client/src/physics/Prediction.ts`
**Acceptance**:
- Log correction frequency (% of snapshots requiring correction)
- Log correction magnitude (average meters of correction)
- Debug overlay showing stats
- Warning if correction frequency > 5% or magnitude > 0.5m
- Test: metrics correctly calculated from test data

#### M6-S10: Multiplayer race integration
**Complexity**: Medium
**Depends on**: All M6 stories
**Acceptance**:
- Two browser tabs can: create room, join, ready, start race
- Both see each other's cars moving in real-time
- Race state synchronized (countdown, laps, finish)
- Disconnect during race: car removed, others continue
- Results show both players' times
- **PLAYTEST**: Is multiplayer fun? Does it feel responsive?

**M6 Definition of Done**: Two browser tabs race each other smoothly on a generated map. Corrections are rare and invisible.

---

## M7: Game Modes (v0.7.0)

**Goal**: All four game modes playable.

### Stories

#### M7-S1: Game mode interface
**Complexity**: Small
**Files**: `client/src/gameplay/GameMode.ts`, `server/crates/rmm-game/src/game_mode.rs`
**Acceptance**:
- GameMode trait/interface: `start()`, `tick()`, `isFinished()`, `getResults()`
- Mode selected in lobby, communicated to all players before race
- Server and client both implement mode logic

#### M7-S2: Circuit mode (refactor from M5)
**Complexity**: Small
**Files**: `client/src/gameplay/Circuit.ts`, `server/crates/rmm-game/src/game_mode.rs`
**Acceptance**:
- Configurable laps (1-10, default 3)
- Existing lap/checkpoint logic from M5 wrapped in GameMode interface
- Test: race ends after configured number of laps

#### M7-S3: Sprint mode
**Complexity**: Medium
**Files**: `client/src/gameplay/Sprint.ts`, `server/crates/rmm-game/src/game_mode.rs`
**Acceptance**:
- A→B point-to-point race
- Start at A, checkpoints along route, finish at B
- Race ends when all players finish or timeout
- Works with Sprint track from M3-S4
- Test: race ends when crossing B checkpoint

#### M7-S4: Elimination mode
**Complexity**: Medium
**Files**: `client/src/gameplay/Elimination.ts`, `server/crates/rmm-game/src/game_mode.rs`
**Acceptance**:
- Last-place player eliminated every 30 seconds
- Eliminated players become spectators (camera follows lead car)
- Eliminated cars removed from physics world
- Last car standing wins
- Minimum 2 players required
- Test: elimination timer fires correctly
- Test: correct player eliminated (last place by checkpoint progress)

#### M7-S5: Time Trial mode
**Complexity**: Medium
**Files**: `client/src/gameplay/TimeTrial.ts`
**Acceptance**:
- Solo mode, no opponents
- Records lap times
- Ghost replay: stores position/rotation each tick, replays as translucent car on retry
- Personal best per track (session only, not persistent)
- Test: ghost replay positions match original run

#### M7-S6: Mode selection in lobby
**Complexity**: Small
**Files**: `client/src/network/Lobby.ts`, `server/crates/rmm-game/src/room.rs`
**Acceptance**:
- Host selects mode from dropdown in lobby
- Mode + settings (lap count) communicated to all players
- All modes accessible from lobby

**M7 Definition of Done**: All 4 game modes playable in multiplayer (Elimination, Circuit) and single-player (Time Trial, Sprint, Circuit).

---

## M8: Power-ups and Weapons (v0.8.0)

**Goal**: Items spawn, can be picked up, and affect gameplay.

### Stories

#### M8-S1: Item spawn system
**Complexity**: Medium
**Files**: `server/crates/rmm-game/src/powerups.rs`, `client/src/gameplay/PowerUp.ts`
**Acceptance**:
- Item boxes spawn at track-designated points (from M3-S5)
- Visual: rotating translucent cubes floating above road
- Boxes respawn 10 seconds after pickup
- Server-authoritative (client shows visual optimistically)
- Test: items spawn at correct positions
- Test: items respawn after cooldown

#### M8-S2: Item pickup and assignment
**Complexity**: Medium
**Files**: `server/crates/rmm-game/src/powerups.rs`
**Depends on**: M8-S1
**Acceptance**:
- Car overlapping item box sensor receives random item
- Max 1 item per car (ignore pickup when holding)
- Position-weighted randomization (last place gets better items)
- Item type broadcast to all players
- Test: pickup assigns item
- Test: already holding item = pickup ignored
- Test: last place gets stronger items more often

#### M8-S3: SpeedBoost and Turbo
**Complexity**: Small
**Files**: `server/crates/rmm-game/src/powerups.rs`, `client/src/gameplay/Weapons.ts`
**Acceptance**:
- SpeedBoost: 50% speed increase for 3 seconds
- Turbo: instant burst (higher magnitude, shorter duration)
- Visual: speed lines or motion blur hint
- Test: speed increases for correct duration, then returns to normal

#### M8-S4: Missile
**Complexity**: Medium
**Files**: `server/crates/rmm-game/src/weapons.rs`, `client/src/gameplay/Weapons.ts`
**Acceptance**:
- Fires forward from car
- Slight homing toward nearest car ahead (~15 deg/sec)
- Despawns after 5 seconds or on hit
- Hit: target car spun/pushed
- Visual: small projectile mesh
- Test: missile travels forward
- Test: missile despawns after lifetime

#### M8-S5: Oil Slick
**Complexity**: Medium
**Files**: `server/crates/rmm-game/src/weapons.rs`, `client/src/gameplay/Weapons.ts`
**Acceptance**:
- Dropped behind car as static sensor zone
- Cars entering zone: grip reduced to near-zero for 1.5 seconds
- Visual: dark puddle decal on road
- Lasts 15 seconds then disappears
- Test: entering oil zone reduces grip
- Test: oil zone expires after duration

#### M8-S6: Shield and Shockwave
**Complexity**: Small
**Files**: `server/crates/rmm-game/src/weapons.rs`, `client/src/gameplay/Weapons.ts`
**Acceptance**:
- Shield: 5 seconds of immunity (absorbs next hit), visual: translucent sphere
- Shockwave: pushes all nearby cars away (radius ~20m), visual: expanding ring
- Test: shield absorbs missile hit
- Test: shockwave pushes nearby car

#### M8-S7: Item context sensitivity
**Complexity**: Small
**Files**: `server/crates/rmm-game/src/powerups.rs`
**Acceptance**:
- Suppress certain items on very narrow road segments (<4m)
- Reduce shockwave radius near water/buildings
- Spawn placement avoids tight chokepoints
- Test: narrow road suppresses missile spawns

**M8 Definition of Done**: Items spawn, pickup works, all 6 item types function correctly. Items feel fair on generated tracks (playtest).

---

## M9: UI/UX (v0.9.0)

**Goal**: Full user-facing interface from map selection to results.

### Stories

#### M9-S1: UI framework and screen manager
**Complexity**: Medium
**Files**: `client/src/ui/UIManager.ts`
**Acceptance**:
- DOM-based overlay system on Three.js canvas
- Screen state machine: MainMenu → MapSelector → Lobby → InGame → Results
- Smooth transitions between screens (fade or slide)
- Responsive layout (handles resize, mobile)

#### M9-S2: Map selector with search
**Complexity**: Large
**Files**: `client/src/ui/screens/MapSelector.ts`
**Acceptance**:
- Search bar querying Nominatim (via server proxy, debounced, 1 req/sec)
- Leaflet map with OpenStreetMap tiles
- Draggable/resizable rectangle for bbox selection (500m max enforced)
- "Generate Track" button sends bbox to server
- Loading progress: "Fetching roads...", "Building world...", "Finding route..."
- Track preview overlay on map before committing
- Featured locations for instant play (pre-cached)
- "Random good location" button
- Recent locations list
- Test: search returns results for "London"
- Test: bbox drag enforces size limit

#### M9-S3: Lobby screen
**Complexity**: Medium
**Files**: `client/src/ui/screens/Lobby.ts`
**Acceptance**:
- Player list with names and color indicators
- Car color selection
- Game mode selector (host only)
- Lap count / settings (host only)
- Ready toggle
- Room code display (for sharing)
- Simple text chat
- "Start Race" button (host, when all ready)
- 3D map preview in background

#### M9-S4: In-game HUD (enhanced from M5)
**Complexity**: Small
**Files**: `client/src/ui/hud/HUD.ts`
**Acceptance**:
- Speed, position ("3rd / 8"), lap counter, held item icon, race timer
- Responsive layout for different screen sizes
- Touch-friendly item use button (mobile)

#### M9-S5: Minimap
**Complexity**: Medium
**Files**: `client/src/ui/hud/Minimap.ts`
**Acceptance**:
- 2D canvas overlay in corner
- Track outline with road shapes
- Colored dots for each player (updates each frame)
- Current player highlighted
- Checkpoint markers

#### M9-S6: Results screen (enhanced)
**Complexity**: Small
**Files**: `client/src/ui/screens/Results.ts`
**Acceptance**:
- Standings with finish times
- Best lap time
- Items used, distance driven (simple stats)
- "Race Again" (same track) and "New Map" buttons
- "Back to Lobby" for multiplayer

#### M9-S7: Main menu
**Complexity**: Small
**Files**: `client/src/ui/screens/MainMenu.ts`
**Acceptance**:
- Play (quick start with featured location)
- Custom Race (go to map selector)
- Join Room (enter room code)
- Settings (placeholder)

**M9 Definition of Done**: Full flow from main menu → pick location → lobby → race → results. Under 5 clicks to first race via featured location.

---

## M10: Discord Activity Integration (v0.10.0)

**Goal**: Game works as a Discord Activity.

### Stories

#### M10-S1: Discord SDK initialization
**Complexity**: Medium
**Files**: `client/src/discord.ts`
**Acceptance**:
- Detects Discord iframe environment
- Initializes DiscordSDK with client ID
- OAuth2 authorization flow (code → server exchange → token)
- Falls back to standalone mode cleanly if not in Discord
- DiscordSDKMock for local development

#### M10-S2: Server-side Discord OAuth
**Complexity**: Medium
**Files**: `server/crates/rmm-auth/src/discord.rs`, `server/crates/rmm-auth/src/standalone.rs`
**Acceptance**:
- Token exchange endpoint (`POST /api/auth/discord`)
- Validates token with Discord API
- Returns user info (id, username, avatar)
- Standalone auth: username → session token (simple, no password)
- Test: valid code exchanges for token
- Test: invalid code returns error

#### M10-S3: Participant tracking
**Complexity**: Small
**Files**: `client/src/discord.ts`
**Acceptance**:
- Subscribe to ACTIVITY_INSTANCE_PARTICIPANTS_UPDATE
- Auto-update lobby when voice channel participants change
- First player creates room, others auto-join

#### M10-S4: Discord proxy configuration
**Complexity**: Medium
**Files**: Documentation + Discord Developer Portal config
**Acceptance**:
- URL mappings configured for: server API, WebSocket, tile server
- `patchUrlMappings()` called for third-party libraries
- All API calls work through Discord proxy
- Tile map loads in Discord iframe
- Test: game loads and connects in Discord Activity mode

#### M10-S5: CSP and iframe compliance
**Complexity**: Small
**Acceptance**:
- No inline scripts in build output
- No eval() usage
- All assets served from same origin
- Cookies set with SameSite=None; Partitioned
- Test: no CSP violations in Discord console

**M10 Definition of Done**: Game launches as Discord Activity, OAuth works, voice channel participants auto-join race.

---

## M11: Polish (v0.11.0)

**Goal**: Audio, visual effects, performance optimization.

### Stories

#### M11-S1: Audio system
**Complexity**: Medium
**Files**: `client/src/utils/AudioManager.ts`
**Acceptance**:
- Web Audio API based
- Engine sound: oscillator with playback rate tied to speed
- Collision SFX on Rapier contact events
- Drift SFX when lateral speed > threshold
- Weapon SFX (missile, oil, shield, shockwave)
- Muted by default, unmute on first interaction
- Spatial panning based on other cars' positions

#### M11-S2: Visual effects
**Complexity**: Medium
**Files**: `client/src/vehicles/CarEffects.ts`
**Acceptance**:
- Tire mark trails (Line geometry following rear wheels during drift)
- Drift smoke (instanced billboard quads, pooled)
- Collision sparks (particle burst at contact point)
- Speed boost visual (motion lines or screen effect)
- Missile explosion (particle burst)
- All particle systems use object pooling

#### M11-S3: Performance optimization
**Complexity**: Medium
**Acceptance**:
- LOD: buildings beyond 200m simplified or culled
- Blob shadows for cars (dark circle under car, not shadow maps)
- Texture atlasing for buildings/roads
- Frustum culling (Three.js automatic)
- Target: 60fps on mid-range hardware
- Performance auto-detection: measure frame time over 60 frames, adjust quality

#### M11-S4: Mobile support
**Complexity**: Medium
**Files**: `client/src/core/Input.ts`
**Acceptance**:
- Virtual joystick for steering (touch)
- Tap zones for throttle, brake, item use
- Reduced quality preset for mobile (fewer particles, lower resolution)
- Touch targets minimum 44x44px
- Test: playable on mobile browser and Discord mobile

#### M11-S5: Soak testing
**Complexity**: Small
**Acceptance**:
- 10 consecutive races in one browser tab: no memory growth
- No detectable leaks in physics objects, meshes, or audio
- Room create/destroy cycle: server memory stable

**M11 Definition of Done**: Audio, particles, 60fps on mid-range hardware, no memory leaks, mobile playable.

---

## M12: Production Release (v1.0.0)

**Goal**: Ship it.

### Stories

#### M12-S1: Deployment infrastructure
- Fly.io config for Rust server
- Static hosting for client (Vercel/Netlify/CDN)
- Environment configuration for production
- SSL/HTTPS

#### M12-S2: Discord App Store submission
- App description, screenshots, promotional images
- Privacy policy URL
- Terms of service URL
- Discord review process

#### M12-S3: Monitoring and analytics
- Server health monitoring
- Error tracking (Sentry or similar)
- Basic analytics: games played, locations used, modes played
- Performance metrics dashboard

#### M12-S4: Final QA
- Full regression across all milestones
- Cross-browser testing (Chrome, Firefox, Safari, Edge)
- Mobile testing (iOS Safari, Android Chrome, Discord mobile)
- Load testing: 50 concurrent rooms
- All KPIs green

**M12 Definition of Done**: Deployed, accessible, monitored, Discord App Store approved.
