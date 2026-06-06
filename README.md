# agent-cadence-progress

_Musical cadence as task completion signal — agents signal resolution the same way music does._

## Concept

In music theory, a **cadence** is a chord progression that signals the end of a phrase. Different cadence types communicate different levels of finality:

- **Perfect authentic** (V → I): "Done. Period. No question."
- **Plagal** (IV → I): "Done, but softly — the 'Amen' ending."
- **Deceptive** (V → vi): "Wait, I thought we were done? Nope, twist!"
- **Half** (→ V): "Hold on, we're paused mid-thought..."
- **Phrygian** (↓v → V): "Dramatic cliffhanger!"

`agent-cadence-progress` maps these patterns to task completion signals. Instead of simple percentage-complete bars, agents use cadence patterns to communicate *how complete* they are — and whether that completion is real or deceptive.

## Key Types

### `CadenceType`

The five musical cadence types, each with a resolution strength (0.0–1.0) and a human-readable description.

```rust
use agent_cadence_progress::CadenceType;

assert!(CadenceType::PerfectAuthentic.is_resolved());
assert!(!CadenceType::Deceptive.is_resolved());
assert!(CadenceType::PerfectAuthentic.resolution_strength() > CadenceType::Plagal.resolution_strength());
```

### `TaskProgress`

Maps a task's numeric progress to a cadence type. Progress values translate to cadences:

| Progress Range | Cadence | Meaning |
|---|---|---|
| 0.98–1.0 | Perfect Authentic | Definitively complete |
| 0.90–0.97 | Plagal | Complete, soft landing |
| 0.80–0.89 | Deceptive | Looks done but isn't |
| 0.40–0.79 | Half | In progress, paused |
| < 0.40 | Phrygian | Just started, tense |

```rust
use agent_cadence_progress::TaskProgress;

let mut task = TaskProgress::with_progress("deploy-api", 0.99);
let cadence = task.detect_cadence();
assert!(task.is_complete());
```

### `CompletionSignal`

Detects when a group of agents collectively reaches a cadence point. Aggregates individual task progresses to determine the group's overall completion signal.

```rust
use agent_cadence_progress::{TaskProgress, CompletionSignal};

let tasks = vec![
    TaskProgress::with_progress("frontend", 0.98),
    TaskProgress::with_progress("backend", 1.0),
    TaskProgress::with_progress("database", 0.99),
];
let signal = CompletionSignal::detect_from_group(&tasks, 1).unwrap();
assert!(signal.is_final());
```

### `DeceptiveResolution`

Detects tasks that appear complete but regress — the "deceptive cadence" of task management. Tracks when progress goes backwards, indicating a false completion.

```rust
use agent_cadence_progress::DeceptiveResolution;

let mut dr = DeceptiveResolution::new(0.05);
dr.track("auth-service", 0.9, 0.5, 2); // Regression detected!
assert!(dr.is_deceptive("auth-service"));
```

### Chord Functions and `ProgressionTracker`

Models task progression as a chord progression using standard harmonic functions:

- **Tonic (I)**: Home base, stable — task begins or completes
- **Supertonic (ii)**: Moving away — gathering resources
- **Mediant (iii)**: Ambiguous — exploring options
- **Subdominant (IV)**: Building tension — work in progress
- **Dominant (V)**: Maximum tension — approaching resolution
- **Submediant (vi)**: Deceptive resolution — false finish
- **Leading (vii°)**: Highly unstable — critical moment

```rust
use agent_cadence_progress::ProgressionTracker;

let mut tracker = ProgressionTracker::standard();
// I → ii → IV → V → I
while let Some(chord) = tracker.advance() {
    println!("Tension: {:.2}", chord.tension());
}
let cadence = tracker.detect_cadence(); // PerfectAuthentic
```

### `CadenceCoordinator`

Orchestrates multiple tasks with a shared progression tracker.

## Why Cadences?

Simple percentage-complete metrics lose nuance. A task at 85% might be nearly done (building toward authentic cadence) or might be about to fail (building toward deceptive cadence). Musical cadence theory gives us a richer vocabulary for understanding completion states.

## Installation

```toml
[dependencies]
agent-cadence-progress = { path = "." }
```

## License

MIT
