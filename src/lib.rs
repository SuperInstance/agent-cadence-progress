//! # agent-cadence-progress
//!
//! Musical cadence as task completion signal. Musical cadences signal resolution
//! and finality — agents can signal task completion using the same patterns.

use std::collections::HashMap;

/// Types of musical cadence mapped to task completion states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CadenceType {
    /// Perfect authentic cadence (V → I) — fully complete, no ambiguity
    PerfectAuthentic,
    /// Plagal cadence (IV → I) — complete but softer, "amen" ending
    Plagal,
    /// Deceptive cadence (V → vi) — looks done but isn't, surprise twist
    Deceptive,
    /// Half cadence (→ V) — paused mid-way, expects continuation
    Half,
    /// Phrygian half cadence (↓v → V) — tense pause, dramatic interruption
    Phrygian,
}

impl CadenceType {
    /// Human-readable description of the cadence.
    pub fn description(&self) -> &str {
        match self {
            Self::PerfectAuthentic => "Full resolution — task definitively complete",
            Self::Plagal => "Gentle resolution — task complete with soft landing",
            Self::Deceptive => "False resolution — task appears done but isn't",
            Self::Half => "Suspension — task paused, awaiting continuation",
            Self::Phrygian => "Dramatic pause — task interrupted with tension",
        }
    }

    /// Is this cadence a final resolution?
    pub fn is_resolved(&self) -> bool {
        matches!(self, Self::PerfectAuthentic | Self::Plagal)
    }

    /// Resolution strength from 0.0 (no resolution) to 1.0 (complete).
    pub fn resolution_strength(&self) -> f64 {
        match self {
            Self::PerfectAuthentic => 1.0,
            Self::Plagal => 0.85,
            Self::Deceptive => 0.3,
            Self::Half => 0.1,
            Self::Phrygian => 0.05,
        }
    }
}

/// Task progress mapped from cadence patterns.
#[derive(Debug, Clone)]
pub struct TaskProgress {
    task_id: String,
    progress: f64, // 0.0..1.0
    cadence: Option<CadenceType>,
}

impl TaskProgress {
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            progress: 0.0,
            cadence: None,
        }
    }

    pub fn with_progress(task_id: impl Into<String>, progress: f64) -> Self {
        Self {
            task_id: task_id.into(),
            progress: progress.clamp(0.0, 1.0),
            cadence: None,
        }
    }

    /// Map a progress value to the most likely cadence type.
    pub fn detect_cadence(&mut self) -> CadenceType {
        let cadence = if self.progress >= 0.98 {
            CadenceType::PerfectAuthentic
        } else if self.progress >= 0.90 {
            CadenceType::Plagal
        } else if self.progress >= 0.70 {
            // Ambiguous zone — could be deceptive
            if self.progress >= 0.80 {
                CadenceType::Deceptive
            } else {
                CadenceType::Half
            }
        } else if self.progress >= 0.40 {
            CadenceType::Half
        } else {
            CadenceType::Phrygian
        };
        self.cadence = Some(cadence);
        cadence
    }

    pub fn set_progress(&mut self, p: f64) {
        self.progress = p.clamp(0.0, 1.0);
    }

    pub fn progress(&self) -> f64 {
        self.progress
    }

    pub fn cadence(&self) -> Option<CadenceType> {
        self.cadence
    }

    pub fn task_id(&self) -> &str {
        &self.task_id
    }

    pub fn is_complete(&self) -> bool {
        self.cadence.map_or(false, |c| c.is_resolved())
    }
}

/// Signal when a group of agents reaches a cadence point.
#[derive(Debug, Clone)]
pub struct CompletionSignal {
    /// The cadence type reached
    pub cadence: CadenceType,
    /// Agents that contributed to the signal
    pub agents: Vec<String>,
    /// Overall completion percentage
    pub overall_completion: f64,
    /// Timestamp or step number
    pub step: u64,
}

impl CompletionSignal {
    /// Detect completion signals from a group of task progresses.
    pub fn detect_from_group(tasks: &[TaskProgress], step: u64) -> Option<Self> {
        if tasks.is_empty() {
            return None;
        }

        let total: f64 = tasks.iter().map(|t| t.progress()).sum();
        let avg = total / tasks.len() as f64;

        // Determine group cadence based on average progress
        let cadence = if avg >= 0.98 {
            CadenceType::PerfectAuthentic
        } else if avg >= 0.90 {
            CadenceType::Plagal
        } else if avg >= 0.75 {
            CadenceType::Deceptive
        } else if avg >= 0.40 {
            CadenceType::Half
        } else {
            CadenceType::Phrygian
        };

        // Only emit signal for meaningful cadences
        if matches!(cadence, CadenceType::Phrygian) && avg < 0.1 {
            return None;
        }

        let agents: Vec<String> = tasks.iter().map(|t| t.task_id().to_string()).collect();

        Some(Self {
            cadence,
            agents,
            overall_completion: avg,
            step,
        })
    }

    /// Is this a final completion signal?
    pub fn is_final(&self) -> bool {
        self.cadence.is_resolved()
    }
}

/// Detects when tasks look done but aren't (deceptive cadence).
#[derive(Debug, Clone)]
pub struct DeceptiveResolution {
    /// Tasks that appeared complete but regressed
    regressions: Vec<RegressionEvent>,
    /// Threshold for detecting a regression
    regression_threshold: f64,
}

#[derive(Debug, Clone)]
struct RegressionEvent {
    task_id: String,
    apparent_progress: f64,
    actual_progress: f64,
    step: u64,
}

impl DeceptiveResolution {
    pub fn new(regression_threshold: f64) -> Self {
        Self {
            regressions: Vec::new(),
            regression_threshold,
        }
    }

    /// Track progress and detect if a task regresses (deceptive resolution).
    pub fn track(&mut self, task_id: impl Into<String>, previous: f64, current: f64, step: u64) -> bool {
        let regression = previous - current;
        if regression > self.regression_threshold {
            self.regressions.push(RegressionEvent {
                task_id: task_id.into(),
                apparent_progress: previous,
                actual_progress: current,
                step,
            });
            true
        } else {
            false
        }
    }

    /// Number of regressions detected.
    pub fn regression_count(&self) -> usize {
        self.regressions.len()
    }

    /// Get all tasks that had deceptive resolutions.
    pub fn deceptive_tasks(&self) -> Vec<&str> {
        self.regressions.iter().map(|r| r.task_id.as_str()).collect()
    }

    /// Check if a task had a deceptive resolution.
    pub fn is_deceptive(&self, task_id: &str) -> bool {
        self.regressions.iter().any(|r| r.task_id == task_id)
    }
}

/// Chord progressions mapped to task progressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChordFunction {
    Tonic,      // I — home base, stable
    Supertonic, // ii — pre-dominant, moving away
    Mediant,    // iii — ambiguous
    Subdominant,// IV — pre-dominant, building tension
    Dominant,   // V — tension, expecting resolution
    Submediant, // vi — relative minor, deceptive resolution
    Leading,    // vii° — diminished, highly unstable
}

impl ChordFunction {
    /// Stability of the chord function (0.0 = unstable, 1.0 = stable).
    pub fn stability(&self) -> f64 {
        match self {
            Self::Tonic => 1.0,
            Self::Submediant => 0.7,
            Self::Subdominant => 0.5,
            Self::Mediant => 0.4,
            Self::Supertonic => 0.3,
            Self::Dominant => 0.15,
            Self::Leading => 0.05,
        }
    }

    /// Tension level (inverse of stability).
    pub fn tension(&self) -> f64 {
        1.0 - self.stability()
    }
}

/// Tracker that models task progression as a chord progression.
#[derive(Debug, Clone)]
pub struct ProgressionTracker {
    /// The progression of chord functions for each step
    progression: Vec<ChordFunction>,
    /// Current position in the progression
    position: usize,
}

impl ProgressionTracker {
    pub fn new() -> Self {
        Self {
            progression: Vec::new(),
            position: 0,
        }
    }

    /// Create a standard I-IV-V-I progression (common task flow).
    pub fn standard() -> Self {
        Self {
            progression: vec![
                ChordFunction::Tonic,      // Start: task begins
                ChordFunction::Supertonic,  // Setup: gathering resources
                ChordFunction::Subdominant, // Build: working
                ChordFunction::Dominant,    // Climax: approaching completion
                ChordFunction::Tonic,       // Resolve: task complete
            ],
            position: 0,
        }
    }

    /// Create a deceptive progression (I-IV-V-vi).
    pub fn deceptive() -> Self {
        Self {
            progression: vec![
                ChordFunction::Tonic,
                ChordFunction::Subdominant,
                ChordFunction::Dominant,
                ChordFunction::Submediant, // Deceptive!
                ChordFunction::Subdominant, // Try again
                ChordFunction::Dominant,
                ChordFunction::Tonic,       // Finally resolve
            ],
            position: 0,
        }
    }

    /// Advance to the next step in the progression.
    pub fn advance(&mut self) -> Option<ChordFunction> {
        if self.position < self.progression.len() {
            let chord = self.progression[self.position];
            self.position += 1;
            Some(chord)
        } else {
            None
        }
    }

    /// Get the current chord function.
    pub fn current(&self) -> Option<&ChordFunction> {
        self.progression.get(self.position.saturating_sub(1))
    }

    /// Detect the cadence type from the last two chords.
    pub fn detect_cadence(&self) -> Option<CadenceType> {
        if self.position < 2 {
            return None;
        }
        let prev = self.progression[self.position - 2];
        let curr = self.progression[self.position - 1];
        Some(match (prev, curr) {
            (ChordFunction::Dominant, ChordFunction::Tonic) => CadenceType::PerfectAuthentic,
            (ChordFunction::Subdominant, ChordFunction::Tonic) => CadenceType::Plagal,
            (ChordFunction::Dominant, ChordFunction::Submediant) => CadenceType::Deceptive,
            (ChordFunction::Supertonic, ChordFunction::Dominant) => CadenceType::Phrygian,
            (_, ChordFunction::Dominant) => CadenceType::Half,
            _ => CadenceType::Half, // Default to half for unclear endings
        })
    }

    /// Current tension level of the progression.
    pub fn tension(&self) -> f64 {
        self.current().map(|c| c.tension()).unwrap_or(0.0)
    }

    /// Progress through the progression as a ratio.
    pub fn progress_ratio(&self) -> f64 {
        if self.progression.is_empty() {
            0.0
        } else {
            self.position as f64 / self.progression.len() as f64
        }
    }

    pub fn is_complete(&self) -> bool {
        self.position >= self.progression.len()
    }

    pub fn progression(&self) -> &[ChordFunction] {
        &self.progression
    }
}

/// A multi-agent cadence coordinator.
#[derive(Debug, Clone)]
pub struct CadenceCoordinator {
    tasks: HashMap<String, TaskProgress>,
    tracker: ProgressionTracker,
}

impl CadenceCoordinator {
    pub fn new(tracker: ProgressionTracker) -> Self {
        Self {
            tasks: HashMap::new(),
            tracker,
        }
    }

    pub fn add_task(&mut self, task: TaskProgress) {
        self.tasks.insert(task.task_id().to_string(), task);
    }

    pub fn update_task(&mut self, task_id: &str, progress: f64) {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.set_progress(progress);
        }
    }

    /// Check for group cadence at the current step.
    pub fn check_cadence(&mut self, step: u64) -> Option<CompletionSignal> {
        let tasks: Vec<TaskProgress> = self.tasks.values().cloned().collect();
        CompletionSignal::detect_from_group(&tasks, step)
    }

    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cadence_type_properties() {
        assert!(CadenceType::PerfectAuthentic.is_resolved());
        assert!(CadenceType::Plagal.is_resolved());
        assert!(!CadenceType::Deceptive.is_resolved());
        assert!(!CadenceType::Half.is_resolved());
        assert!(!CadenceType::Phrygian.is_resolved());

        assert!(CadenceType::PerfectAuthentic.resolution_strength() > CadenceType::Plagal.resolution_strength());
        assert!(CadenceType::Plagal.resolution_strength() > CadenceType::Deceptive.resolution_strength());
    }

    #[test]
    fn test_cadence_descriptions() {
        for ct in [
            CadenceType::PerfectAuthentic,
            CadenceType::Plagal,
            CadenceType::Deceptive,
            CadenceType::Half,
            CadenceType::Phrygian,
        ] {
            assert!(!ct.description().is_empty());
        }
    }

    #[test]
    fn test_task_progress_detect_cadence() {
        let mut tp = TaskProgress::with_progress("t1", 0.99);
        assert_eq!(tp.detect_cadence(), CadenceType::PerfectAuthentic);

        let mut tp = TaskProgress::with_progress("t2", 0.92);
        assert_eq!(tp.detect_cadence(), CadenceType::Plagal);

        let mut tp = TaskProgress::with_progress("t3", 0.83);
        assert_eq!(tp.detect_cadence(), CadenceType::Deceptive);

        let mut tp = TaskProgress::with_progress("t4", 0.50);
        assert_eq!(tp.detect_cadence(), CadenceType::Half);

        let mut tp = TaskProgress::with_progress("t5", 0.10);
        assert_eq!(tp.detect_cadence(), CadenceType::Phrygian);
    }

    #[test]
    fn test_task_progress_completion() {
        let mut tp = TaskProgress::new("t1");
        assert!(!tp.is_complete());
        tp.set_progress(1.0);
        tp.detect_cadence();
        assert!(tp.is_complete());
    }

    #[test]
    fn test_task_progress_clamp() {
        let tp = TaskProgress::with_progress("t1", 1.5);
        assert!((tp.progress() - 1.0).abs() < 1e-9);
        let tp = TaskProgress::with_progress("t2", -0.5);
        assert!((tp.progress()).abs() < 1e-9);
    }

    #[test]
    fn test_completion_signal_from_group() {
        let tasks = vec![
            TaskProgress::with_progress("a", 0.99),
            TaskProgress::with_progress("b", 0.98),
            TaskProgress::with_progress("c", 1.0),
        ];
        let signal = CompletionSignal::detect_from_group(&tasks, 1).unwrap();
        assert!(signal.is_final());
        assert_eq!(signal.agents.len(), 3);
        assert!(signal.overall_completion > 0.95);
    }

    #[test]
    fn test_completion_signal_deceptive() {
        let tasks = vec![
            TaskProgress::with_progress("a", 0.80),
            TaskProgress::with_progress("b", 0.75),
        ];
        let signal = CompletionSignal::detect_from_group(&tasks, 1).unwrap();
        assert_eq!(signal.cadence, CadenceType::Deceptive);
    }

    #[test]
    fn test_completion_signal_empty() {
        let tasks: Vec<TaskProgress> = vec![];
        assert!(CompletionSignal::detect_from_group(&tasks, 1).is_none());
    }

    #[test]
    fn test_deceptive_resolution_detection() {
        let mut dr = DeceptiveResolution::new(0.05);
        assert!(!dr.track("t1", 0.5, 0.6, 1)); // Progress, not regression
        assert!(dr.track("t1", 0.6, 0.3, 2));  // Big regression!
        assert_eq!(dr.regression_count(), 1);
        assert!(dr.is_deceptive("t1"));
        assert!(!dr.is_deceptive("t2"));
    }

    #[test]
    fn test_deceptive_resolution_small_regression() {
        let mut dr = DeceptiveResolution::new(0.2);
        assert!(!dr.track("t1", 0.8, 0.7, 1)); // Small regression below threshold
        assert_eq!(dr.regression_count(), 0);
    }

    #[test]
    fn test_deceptive_tasks_list() {
        let mut dr = DeceptiveResolution::new(0.05);
        dr.track("t1", 0.9, 0.5, 1);
        dr.track("t2", 0.8, 0.4, 2);
        let tasks = dr.deceptive_tasks();
        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn test_chord_function_stability() {
        assert!((ChordFunction::Tonic.stability() - 1.0).abs() < 1e-9);
        assert!((ChordFunction::Leading.stability() - 0.05).abs() < 1e-9);
        assert!((ChordFunction::Dominant.tension() - 0.85).abs() < 1e-9);
    }

    #[test]
    fn test_progression_tracker_standard() {
        let mut tracker = ProgressionTracker::standard();
        assert_eq!(tracker.progression().len(), 5);

        let c1 = tracker.advance();
        assert_eq!(c1, Some(ChordFunction::Tonic));
        assert!(!tracker.is_complete());

        tracker.advance(); // Supertonic
        tracker.advance(); // Subdominant
        tracker.advance(); // Dominant
        let c5 = tracker.advance(); // Tonic
        assert_eq!(c5, Some(ChordFunction::Tonic));
        assert!(tracker.is_complete());
        assert!(tracker.advance().is_none());
    }

    #[test]
    fn test_progression_tracker_cadence_detection() {
        let mut tracker = ProgressionTracker::standard();
        tracker.advance(); // I
        tracker.advance(); // ii
        tracker.advance(); // IV
        tracker.advance(); // V
        assert_eq!(tracker.detect_cadence(), Some(CadenceType::Half)); // IV->V = half
        tracker.advance(); // I
        assert_eq!(tracker.detect_cadence(), Some(CadenceType::PerfectAuthentic));
    }

    #[test]
    fn test_progression_tracker_deceptive() {
        let mut tracker = ProgressionTracker::deceptive();
        tracker.advance(); // I
        tracker.advance(); // IV
        tracker.advance(); // V
        tracker.advance(); // vi — deceptive!
        assert_eq!(tracker.detect_cadence(), Some(CadenceType::Deceptive));
    }

    #[test]
    fn test_progression_tracker_tension() {
        let mut tracker = ProgressionTracker::standard();
        tracker.advance(); // Tonic
        assert!((tracker.tension() - 0.0).abs() < 1e-9);
        tracker.advance(); // Supertonic
        assert!(tracker.tension() > 0.0);
    }

    #[test]
    fn test_progression_tracker_progress_ratio() {
        let mut tracker = ProgressionTracker::new();
        assert!((tracker.progress_ratio()).abs() < 1e-9);
        tracker.progression.push(ChordFunction::Tonic);
        tracker.progression.push(ChordFunction::Dominant);
        tracker.advance();
        assert!((tracker.progress_ratio() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_cadence_coordinator() {
        let tracker = ProgressionTracker::standard();
        let mut coord = CadenceCoordinator::new(tracker);
        coord.add_task(TaskProgress::with_progress("a", 0.0));
        coord.add_task(TaskProgress::with_progress("b", 0.0));
        assert_eq!(coord.task_count(), 2);

        coord.update_task("a", 1.0);
        coord.update_task("b", 1.0);
        let signal = coord.check_cadence(1).unwrap();
        assert!(signal.is_final());
    }
}
