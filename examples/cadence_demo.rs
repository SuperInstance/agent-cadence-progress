use agent_cadence_progress::*;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           🎼 Cadence Types & Resolution Demo 🎼             ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // === Section 1: Cadence Types Overview ===
    println!("━━━ Section 1: The Five Cadence Types ━━━");
    println!();

    let cadences = [
        CadenceType::PerfectAuthentic,
        CadenceType::Plagal,
        CadenceType::Deceptive,
        CadenceType::Half,
        CadenceType::Phrygian,
    ];

    println!("  {:>20}  {:>8}  {:>10}  {}", "Cadence", "Resolved?", "Strength", "Description");
    println!("  {}  {}  {}  {}", "────────────────────", "────────", "──────────", "────────────────────────────────────────");
    for c in &cadences {
        println!("  {:>20}  {:>8}  {:>9.0}%  {}",
            format!("{:?}", c),
            if c.is_resolved() { "✅ Yes" } else { "❌ No" },
            c.resolution_strength() * 100.0,
            c.description()
        );
    }
    println!();

    // === Section 2: Progress through cadences ===
    println!("━━━ Section 2: Task Progress → Cadence Mapping ━━━");
    println!();

    let progressions = [
        (0.05, "Just started — confused rummaging"),
        (0.25, "Getting oriented"),
        (0.50, "Halfway — the real work begins"),
        (0.65, "Building momentum"),
        (0.75, "Almost there... or is it?"),
        (0.82, "Deceptive! Thought we were done"),
        (0.90, "Soft landing — we made it"),
        (0.99, "Perfect completion!"),
        (1.00, "Definitive resolution"),
    ];

    println!("  {:>6}  {:>20}  {:>10}  {}", "Prog%", "Cadence", "Strength", "Mood");
    println!("  {}  {}  {}  {}", "─────", "────────────────────", "──────────", "────────────────────────────────");
    for (prog, mood) in &progressions {
        let mut tp = TaskProgress::with_progress("demo", *prog);
        let cadence = tp.detect_cadence();
        println!("  {:>5.0}%  {:>20}  {:>9.0}%  {}",
            prog * 100.0,
            format!("{:?}", cadence),
            cadence.resolution_strength() * 100.0,
            mood
        );
    }
    println!();

    // === Section 3: Chord Progression as Task Flow ===
    println!("━━━ Section 3: Standard Progression (I → ii → IV → V → I) ━━━");
    println!();

    let mut tracker = ProgressionTracker::standard();
    println!("  Mapping task stages to chord functions:");
    println!();

    let stage_names = [
        "Task begins — stable start",
        "Setup phase — gathering resources",
        "Building — main work in progress",
        "Climax — approaching completion",
        "Resolution — task complete!",
    ];

    println!("  {:>5}  {:>12}  {:>8}  {:>10}  {:>6}  {}", "Step", "Chord", "Stability", "Tension", "Prog%", "Stage");
    println!("  {}  {}  {}  {}  {}  {}", "─────", "────────────", "────────", "──────────", "─────", "────────────────────────────────────");

    for (i, stage) in stage_names.iter().enumerate() {
        let chord = tracker.advance().unwrap();
        let cadence = tracker.detect_cadence();
        println!("  {:>5}  {:>12}  {:>8.0}%  {:>9.0}%  {:>5.0}%  {}",
            i + 1,
            format!("{:?}", chord),
            chord.stability() * 100.0,
            chord.tension() * 100.0,
            tracker.progress_ratio() * 100.0,
            stage
        );
        if let Some(c) = cadence {
            println!("        └─ Cadence detected: {:?} (strength: {:.0}%)", c, c.resolution_strength() * 100.0);
        }
    }
    println!();

    // === Section 4: Deceptive Progression ===
    println!("━━━ Section 4: Deceptive Progression (I → IV → V → vi ✗ → IV → V → I) ━━━");
    println!();

    let mut deceptive = ProgressionTracker::deceptive();
    let deceptive_labels = [
        "Start",
        "Building",
        "Climax — here it comes!",
        "WAIT — deceptive! Not done yet!",
        "Regroup",
        "Try again",
        "Finally resolved",
    ];

    println!("  {:>5}  {:>12}  {:>10}  {}", "Step", "Chord", "Cadence?", "What happened");
    println!("  {}  {}  {}  {}", "─────", "────────────", "──────────", "─────────────────────────────────");
    for (i, label) in deceptive_labels.iter().enumerate() {
        let chord = deceptive.advance().unwrap();
        let cadence = deceptive.detect_cadence();
        let cadence_str = cadence.map(|c| format!("{:?}", c)).unwrap_or("—".to_string());
        let marker = match cadence {
            Some(CadenceType::Deceptive) => "⚠️",
            Some(CadenceType::PerfectAuthentic) => "✅",
            _ => "  ",
        };
        println!("  {:>5}  {:>12}  {:>10}{} {}", i + 1, format!("{:?}", chord), cadence_str, marker, label);
    }
    println!();

    // === Section 5: Deceptive Resolution Detection ===
    println!("━━━ Section 5: Detecting Deceptive Resolutions (Task Regressions) ━━━");
    println!();

    let mut dr = DeceptiveResolution::new(0.05);
    println!("  Tracking task 'deploy-app' through progress updates:");
    println!();

    let updates = [
        (0.10, "Initial setup"),
        (0.30, "Dependencies installed"),
        (0.55, "Tests passing"),
        (0.80, "Deploying to staging"),
        (0.95, "Looks good!"),
        (0.60, "💀 Rollback — tests failed in production!"),
        (0.65, "Fixing the issue"),
        (0.85, "Re-deploying"),
        (0.95, "All green now"),
        (1.00, "Fully deployed ✓"),
    ];

    let mut prev = 0.0;
    for (step, (progress, desc)) in updates.iter().enumerate() {
        let is_deceptive = dr.track("deploy-app", prev, *progress, step as u64);
        let marker = if is_deceptive { "⚠️ DECEPTIVE REGRESSION!" } else { "" };
        println!("  Step {:>2}: {:>5.0}% → {:>5.0}%  {}  {}", step, prev * 100.0, progress * 100.0, desc, marker);
        prev = *progress;
    }

    println!();
    println!("  Total regressions detected: {}", dr.regression_count());
    println!("  Deceptive tasks: {:?}", dr.deceptive_tasks());
    println!();

    // === Section 6: Multi-Agent Completion ===
    println!("━━━ Section 6: Multi-Agent Completion Signals ━━━");
    println!();

    let mut coord = CadenceCoordinator::new(ProgressionTracker::standard());
    coord.add_task(TaskProgress::with_progress("frontend", 0.0));
    coord.add_task(TaskProgress::with_progress("backend", 0.0));
    coord.add_task(TaskProgress::with_progress("database", 0.0));

    let updates_by_step = [
        vec![("frontend", 0.2), ("backend", 0.15), ("database", 0.3)],
        vec![("frontend", 0.5), ("backend", 0.45), ("database", 0.6)],
        vec![("frontend", 0.8), ("backend", 0.7), ("database", 0.85)],
        vec![("frontend", 0.95), ("backend", 0.82), ("database", 0.97)],
        vec![("frontend", 1.0), ("backend", 1.0), ("database", 1.0)],
    ];

    for (step, updates) in updates_by_step.iter().enumerate() {
        for (task, progress) in updates {
            coord.update_task(task, *progress);
        }
        if let Some(signal) = coord.check_cadence(step as u64) {
            println!("  Step {}: Avg={:.0}% | Cadence={:?} | Final={} | Agents={:?}",
                step, signal.overall_completion * 100.0,
                signal.cadence,
                if signal.is_final() { "✅" } else { "..." },
                signal.agents
            );
        }
    }

    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║    Every ending is a new beginning in music 🎶               ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}
