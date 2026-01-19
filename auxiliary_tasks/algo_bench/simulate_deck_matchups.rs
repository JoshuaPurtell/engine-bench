use std::collections::HashMap;
use std::io::{self, Write};
use std::thread;
use tcg_ai::{AiController, RandomAiV4};
use tcg_core::{Action, CardInstance, CardMetaMap, GameState, PlayerId, StepResult};
use tcg_rules_ex::RulesetConfig;

mod ref_random_ai {
    include!("reference_algos/random_ai.rs");
}

mod ref_random_ai_v2 {
    include!("reference_algos/random_ai_v2.rs");
}

mod ref_random_ai_v3 {
    include!("reference_algos/random_ai_v3.rs");
}

use ref_random_ai::RandomAi;
use ref_random_ai_v2::RandomAiV2;
use ref_random_ai_v3::RandomAiV3;

#[derive(PartialEq, Clone, Copy, Debug)]
enum AiType {
    RandomAi,
    RandomAiV2,
    RandomAiV3,
    RandomAiV4,
}

/// Guard that redirects stderr to /dev/null and restores it on drop
#[cfg(unix)]
struct StderrGuard {
    saved_fd: i32,
}

#[cfg(unix)]
impl StderrGuard {
    fn new() -> Self {
        use std::os::unix::io::AsRawFd;
        let null = std::fs::File::create("/dev/null").expect("Failed to open /dev/null");
        let null_fd = null.as_raw_fd();
        let stderr_fd = io::stderr().as_raw_fd();
        // Save original stderr
        let saved_fd = unsafe { libc::dup(stderr_fd) };
        // Redirect stderr to /dev/null
        unsafe {
            libc::dup2(null_fd, stderr_fd);
        }
        Self { saved_fd }
    }
}

#[cfg(unix)]
impl Drop for StderrGuard {
    fn drop(&mut self) {
        use std::os::unix::io::AsRawFd;
        unsafe {
            let stderr_fd = io::stderr().as_raw_fd();
            libc::dup2(self.saved_fd, stderr_fd);
            libc::close(self.saved_fd);
        }
    }
}

#[cfg(not(unix))]
struct StderrGuard;

#[cfg(not(unix))]
impl StderrGuard {
    fn new() -> Self {
        Self
    }
}

/// Tries candidate actions in order until one is accepted. Returns true if something applied.
fn apply_first_accepted(game: &mut GameState, player: PlayerId, candidates: Vec<Action>) -> bool {
    for action in candidates {
        if game.apply_action(player, action).is_ok() {
            return true;
        }
    }
    false
}

/// Drive the engine similarly to the server:
/// - repeatedly call `step()`
/// - resolve prompts with the appropriate controller
/// - when `step()` returns Continue in Main/Attack, force an action from the current player
fn run_match_loop(
    mut game: GameState,
    mut p1_ai: Option<&mut dyn AiController>,
    mut p2_ai: Option<&mut dyn AiController>,
    max_steps: usize,
) -> Option<PlayerId> {
    run_match_loop_with_stats(game, p1_ai, p2_ai, max_steps).map(|(winner, _)| winner)
}

fn run_match_loop_with_stats(
    mut game: GameState,
    mut p1_ai: Option<&mut dyn AiController>,
    mut p2_ai: Option<&mut dyn AiController>,
    max_steps: usize,
) -> Option<(PlayerId, GameState)> {
    let mut steps_left = max_steps;
    let mut actions_budget = 5_000usize;

    while steps_left > 0 && actions_budget > 0 {
        match game.step() {
            StepResult::Event { .. } => {}
            StepResult::GameOver { winner } => return Some((winner, game)),
            StepResult::Prompt { prompt, for_player } => {
                let view = game.view_for_player(for_player);
                let mut candidates: Vec<Action> = match for_player {
                    PlayerId::P1 => {
                        if let Some(ai) = p1_ai.as_mut() {
                            ai.propose_prompt_response(&view, &prompt)
                        } else {
                            Vec::new()
                        }
                    }
                    PlayerId::P2 => {
                        if let Some(ai) = p2_ai.as_mut() {
                            ai.propose_prompt_response(&view, &prompt)
                        } else {
                            Vec::new()
                        }
                    }
                };
                candidates.push(Action::EndTurn);
                let applied = apply_first_accepted(&mut game, for_player, candidates);
                if !applied {
                    eprintln!("Failed to apply action for prompt: {:?}", prompt);
                    return None;
                }
                actions_budget = actions_budget.saturating_sub(1);
            }
            StepResult::Continue => {
                let phase = game.turn.phase;
                if matches!(phase, tcg_rules_ex::Phase::Main | tcg_rules_ex::Phase::Attack) {
                    let current = game.turn.player;
                    let view = game.view_for_player(current);
                    let mut candidates: Vec<Action> = match current {
                        PlayerId::P1 => {
                            if let Some(ai) = p1_ai.as_mut() {
                                ai.propose_free_actions(&view)
                            } else {
                                Vec::new()
                            }
                        }
                        PlayerId::P2 => {
                            if let Some(ai) = p2_ai.as_mut() {
                                ai.propose_free_actions(&view)
                            } else {
                                Vec::new()
                            }
                        }
                    };
                    candidates.push(Action::EndTurn);
                    let applied = apply_first_accepted(&mut game, current, candidates);
                    if !applied {
                        eprintln!("Failed to apply action for player: {:?}", current);
                        return None;
                    }
                    actions_budget = actions_budget.saturating_sub(1);
                }
            }
        }
        steps_left -= 1;
    }

    None // Game didn't finish
}

#[derive(Default, Clone)]
struct MatchStats {
    p1_wins: usize,
    total: usize,
    durations: Vec<f64>,
    turns: Vec<usize>,
}

impl MatchStats {
    fn merge(&mut self, other: MatchStats) {
        self.p1_wins += other.p1_wins;
        self.total += other.total;
        self.durations.extend(other.durations);
        self.turns.extend(other.turns);
    }
}

fn ai_name(ai_type: AiType) -> &'static str {
    match ai_type {
        AiType::RandomAi => "RandomAi",
        AiType::RandomAiV2 => "RandomAiV2",
        AiType::RandomAiV3 => "RandomAiV3",
        AiType::RandomAiV4 => "RandomAiV4",
    }
}

fn build_ai(ai_type: AiType, seed: u64) -> Box<dyn AiController> {
    match ai_type {
        AiType::RandomAi => Box::new(RandomAi::new(seed)),
        AiType::RandomAiV2 => Box::new(RandomAiV2::new(seed)),
        AiType::RandomAiV3 => Box::new(RandomAiV3::new(seed)),
        AiType::RandomAiV4 => Box::new(RandomAiV4::new(seed)),
    }
}

fn run_match_series(
    deck1: &DeckConfig,
    deck2: &DeckConfig,
    p1_ai_type: AiType,
    p2_ai_type: AiType,
    num_matches: usize,
    seed_base: u64,
    count_player: PlayerId,
    card_meta: &CardMetaMap,
    threads: usize,
    show_progress: bool,
) -> MatchStats {
    let thread_count = threads.max(1).min(num_matches.max(1));
    let mut stats = MatchStats::default();

    if thread_count <= 1 {
        for match_num in 0..num_matches {
            let seed = seed_base + match_num as u64;
            let deck1_cards = (deck1.load_fn)(PlayerId::P1).unwrap();
            let deck2_cards = (deck2.load_fn)(PlayerId::P2).unwrap();

            let game = GameState::new_with_card_meta(
                deck1_cards,
                deck2_cards,
                seed,
                RulesetConfig::default(),
                card_meta.clone(),
            );

            let mut ai1_box = build_ai(p1_ai_type, seed);
            let mut ai2_box = build_ai(p2_ai_type, seed.wrapping_add(9001));

            let start_time = std::time::Instant::now();
            if let Some((winner, final_game)) = run_match_loop_with_stats(
                game,
                Some(ai1_box.as_mut()),
                Some(ai2_box.as_mut()),
                5_000,
            ) {
                let duration = start_time.elapsed().as_secs_f64();
                let turn_count = final_game.turn.number as usize;
                stats.total += 1;
                if winner == count_player {
                    stats.p1_wins += 1;
                }
                stats.durations.push(duration);
                stats.turns.push(turn_count);
            }

            if show_progress {
                print!(".");
                io::stdout().flush().unwrap();
            }
        }
        return stats;
    }

    let chunk = (num_matches + thread_count - 1) / thread_count;
    thread::scope(|scope| {
        let mut handles = Vec::new();
        for t in 0..thread_count {
            let start = t * chunk;
            let end = (start + chunk).min(num_matches);
            if start >= end {
                continue;
            }
            let handle = scope.spawn(move || {
                let mut local = MatchStats::default();
                for match_num in start..end {
                    let seed = seed_base + match_num as u64;
                    let deck1_cards = (deck1.load_fn)(PlayerId::P1).unwrap();
                    let deck2_cards = (deck2.load_fn)(PlayerId::P2).unwrap();

                    let game = GameState::new_with_card_meta(
                        deck1_cards,
                        deck2_cards,
                        seed,
                        RulesetConfig::default(),
                        card_meta.clone(),
                    );

                    let mut ai1_box = build_ai(p1_ai_type, seed);
                    let mut ai2_box = build_ai(p2_ai_type, seed.wrapping_add(9001));

                    let start_time = std::time::Instant::now();
                    if let Some((winner, final_game)) = run_match_loop_with_stats(
                        game,
                        Some(ai1_box.as_mut()),
                        Some(ai2_box.as_mut()),
                        5_000,
                    ) {
                        let duration = start_time.elapsed().as_secs_f64();
                        let turn_count = final_game.turn.number as usize;
                        local.total += 1;
                        if winner == count_player {
                            local.p1_wins += 1;
                        }
                        local.durations.push(duration);
                        local.turns.push(turn_count);
                    }
                }
                local
            });
            handles.push(handle);
        }
        for handle in handles {
            stats.merge(handle.join().unwrap());
        }
    });

    stats
}

fn run_blended_matchup(
    deck1: &DeckConfig,
    deck2: &DeckConfig,
    p1_ai_type: AiType,
    p2_ai_type: AiType,
    num_matches: usize,
    seed_base: u64,
    card_meta: &CardMetaMap,
    threads: usize,
    show_progress: bool,
) -> MatchStats {
    let _stderr_guard = StderrGuard::new();
    let mut stats = run_match_series(
        deck1,
        deck2,
        p1_ai_type,
        p2_ai_type,
        num_matches,
        seed_base,
        PlayerId::P1,
        card_meta,
        threads,
        show_progress,
    );
    let swapped = run_match_series(
        deck1,
        deck2,
        p2_ai_type,
        p1_ai_type,
        num_matches,
        seed_base + 50_000,
        PlayerId::P2,
        card_meta,
        threads,
        show_progress,
    );
    stats.merge(swapped);
    stats
}

fn print_game_stats(stats: &MatchStats) {
    if stats.durations.is_empty() || stats.turns.is_empty() {
        return;
    }

    let mut durations = stats.durations.clone();
    let mut turns = stats.turns.clone();
    durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    turns.sort();

    let n = durations.len();
    let median_idx = n / 2;
    let p90_idx = (n as f64 * 0.9) as usize;

    println!("\nGame Statistics:");
    println!("================");
    println!("\nGame Duration (seconds):");
    println!("  Median:  {:.3}s", durations[median_idx]);
    println!("  P90:     {:.3}s", durations[p90_idx.min(n - 1)]);
    println!("  Max:     {:.3}s", durations[n - 1]);
    println!("\nNumber of Turns:");
    println!("  Median:  {}", turns[median_idx]);
    println!("  P90:     {}", turns[p90_idx.min(n - 1)]);
    println!("  Max:     {}", turns[n - 1]);
    println!("  Total games: {}", n);
}

fn load_theme_deck(deck_path: &str, player: PlayerId) -> Option<Vec<CardInstance>> {
    use std::fs;
    let mut current = std::env::current_dir().ok()?;
    loop {
        let data_theme = current.join("data/theme");
        if data_theme.exists() {
            break;
        }
        if !current.pop() {
            return None;
        }
    }

    let full_path = current.join(deck_path);
    let content = fs::read_to_string(&full_path).ok()?;
    let deck_json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let mut deck = Vec::new();

    if let Some(cards) = deck_json.get("cards").and_then(|c| c.as_array()) {
        for card_entry in cards {
            let def_id = card_entry.get("def_id")?.as_str()?;
            let count = card_entry.get("count").and_then(|c| c.as_u64()).unwrap_or(1) as usize;
            for _ in 0..count {
                deck.push(CardInstance::new(
                    tcg_core::CardDefId::new(def_id.to_string()),
                    player,
                ));
            }
        }
    }

    if let Some(energy) = deck_json.get("energy").and_then(|e| e.as_array()) {
        for energy_entry in energy {
            let energy_type = energy_entry.get("type")?.as_str()?;
            let count = energy_entry.get("count").and_then(|c| c.as_u64()).unwrap_or(0) as usize;
            let energy_id = format!("ENERGY-{}", energy_type.to_uppercase());
            for _ in 0..count {
                deck.push(CardInstance::new(
                    tcg_core::CardDefId::new(energy_id.clone()),
                    player,
                ));
            }
        }
    }

    if deck.is_empty() {
        None
    } else {
        Some(deck)
    }
}

fn load_deck_by_name(server_db_path: &str, deck_name: &str, player: PlayerId) -> Option<Vec<CardInstance>> {
    use rusqlite::Connection;
    let conn = Connection::open(server_db_path).ok()?;
    
    // Try to find by name (case-insensitive, public decks)
    let cards_json: String = conn.query_row(
        "SELECT cards_json FROM decks WHERE LOWER(name) LIKE LOWER(?1) AND is_public = 1 LIMIT 1",
        [&format!("%{}%", deck_name)],
        |row| row.get(0),
    ).ok()?;

    #[derive(serde::Deserialize)]
    struct DeckEntry {
        card_def_id: String,
        count: usize,
    }

    let entries: Vec<DeckEntry> = serde_json::from_str(&cards_json).ok()?;
    let mut deck = Vec::new();
    for entry in entries {
        for _ in 0..entry.count {
            deck.push(CardInstance::new(
                tcg_core::CardDefId::new(entry.card_def_id.clone()),
                player,
            ));
        }
    }

    if deck.is_empty() {
        None
    } else {
        Some(deck)
    }
}

fn load_deck_by_id(server_db_path: &str, deck_id: &str, player: PlayerId) -> Option<Vec<CardInstance>> {
    use rusqlite::Connection;
    let conn = Connection::open(server_db_path).ok()?;

    let cards_json: String = conn.query_row(
        "SELECT cards_json FROM decks WHERE deck_id = ?1",
        [deck_id],
        |row| row.get(0),
    ).ok()?;

    #[derive(serde::Deserialize)]
    struct DeckEntry {
        card_def_id: String,
        count: usize,
    }

    let entries: Vec<DeckEntry> = serde_json::from_str(&cards_json).ok()?;
    let mut deck = Vec::new();
    for entry in entries {
        for _ in 0..entry.count {
            deck.push(CardInstance::new(
                tcg_core::CardDefId::new(entry.card_def_id.clone()),
                player,
            ));
        }
    }

    if deck.is_empty() {
        None
    } else {
        Some(deck)
    }
}

fn load_public_deck_configs(server_db_path: &str) -> Vec<DeckConfig> {
    use rusqlite::Connection;
    let conn = match Connection::open(server_db_path) {
        Ok(conn) => conn,
        Err(_) => return Vec::new(),
    };

    let mut decks = Vec::new();
    let mut stmt = match conn.prepare(
        "SELECT deck_id, name FROM decks WHERE is_public = 1 ORDER BY name"
    ) {
        Ok(stmt) => stmt,
        Err(_) => return decks,
    };

    if let Ok(rows) = stmt.query_map([], |row| {
        let deck_id: String = row.get(0)?;
        let name: String = row.get(1)?;
        Ok((deck_id, name))
    }) {
        for row in rows.flatten() {
            let (deck_id, name) = row;
            let db = server_db_path.to_string();
            let deck_id_clone = deck_id.clone();
            decks.push(DeckConfig {
                name,
                load_fn: Box::new(move |player| load_deck_by_id(&db, &deck_id_clone, player)),
            });
        }
    }

    decks
}

fn load_card_meta(cards_db_path: &str) -> CardMetaMap {
    use rusqlite::Connection;
    let conn = Connection::open(cards_db_path).ok();
    if let Some(conn) = conn {
        tcg_db::load_card_meta_map(&conn).unwrap_or_default()
    } else {
        CardMetaMap::new()
    }
}

struct DeckConfig {
    name: String,
    load_fn: Box<dyn Fn(PlayerId) -> Option<Vec<CardInstance>> + Send + Sync>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let server_db_path = args.get(1).map(|s| s.as_str()).unwrap_or("data/server.sqlite");
    let cards_db_path = args.get(2).map(|s| s.as_str()).unwrap_or("data/cards.sqlite");
    let num_matches = args
        .get(3)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10);
    let p1_ai_type = args.get(4).map(|s| match s.as_str() {
        "v2" => AiType::RandomAiV2,
        "v3" => AiType::RandomAiV3,
        "v4" => AiType::RandomAiV4,
        _ => AiType::RandomAi,
    }).unwrap_or(AiType::RandomAi);
    let maybe_mode = args.get(5).map(|s| s.as_str()).unwrap_or("");
    let mode_in_p2 = matches!(maybe_mode, "single" | "diagonal" | "bench" | "public-bench");
    let p2_ai_type = if mode_in_p2 {
        AiType::RandomAi
    } else {
        args.get(5).map(|s| match s.as_str() {
            "v2" => AiType::RandomAiV2,
            "v3" => AiType::RandomAiV3,
            "v4" => AiType::RandomAiV4,
            _ => AiType::RandomAi,
        }).unwrap_or(AiType::RandomAi)
    };

    let mode = if mode_in_p2 {
        maybe_mode
    } else {
        args.get(6).map(|s| s.as_str()).unwrap_or("")
    };
    let single_deck_matchup = mode == "single";
    let diagonal_matchups = mode == "diagonal";
    let bench_matchups = mode == "bench" || mode == "public-bench";
    let threads = args
        .get(if mode_in_p2 { 6 } else { 7 })
        .and_then(|s| s.parse::<usize>().ok())
        .or_else(|| {
            std::env::var("SIM_THREADS")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
        })
        .unwrap_or(1);

    println!("Deck Matchup Simulator");
    println!("======================");
    println!("Server DB: {}", server_db_path);
    println!("Cards DB: {}", cards_db_path);
    println!("Matches per pairing: {}", num_matches);
    println!(
        "P1 AI: {:?}",
        match p1_ai_type {
            AiType::RandomAi => "RandomAi",
            AiType::RandomAiV2 => "RandomAiV2",
            AiType::RandomAiV3 => "RandomAiV3",
            AiType::RandomAiV4 => "RandomAiV4",
        }
    );
    println!(
        "P2 AI: {:?}",
        match p2_ai_type {
            AiType::RandomAi => "RandomAi",
            AiType::RandomAiV2 => "RandomAiV2",
            AiType::RandomAiV3 => "RandomAiV3",
            AiType::RandomAiV4 => "RandomAiV4",
        }
    );
    if threads > 1 {
        println!("Threads: {}", threads);
    }
    println!();

    // Load card metadata from cards database
    let card_meta = load_card_meta(cards_db_path);
    println!("Loaded {} card definitions", card_meta.len());

    // If single-deck or diagonal matchup with different AIs, we'll blend both scenarios
    let blend_matchups = (single_deck_matchup || diagonal_matchups) && p1_ai_type != p2_ai_type;
    
    // Define decks
    let decks = if bench_matchups {
        load_public_deck_configs(server_db_path)
    } else if single_deck_matchup {
        // Only load Overzealous for AI vs AI matchup
        vec![
            DeckConfig {
                name: "Overzealous-v0".to_string(),
                load_fn: Box::new({
                    let db = server_db_path.to_string();
                    move |player| load_deck_by_name(&db, "Overzealous", player)
                }),
            },
        ]
    } else if diagonal_matchups {
        // Load all decks for diagonal matchups (each vs itself)
        vec![
            DeckConfig {
                name: "Green Cyclone".to_string(),
                load_fn: Box::new(|player| load_theme_deck("data/theme/green_cyclone.json", player)),
            },
            DeckConfig {
                name: "Sceptile ex Delta".to_string(),
                load_fn: Box::new({
                    let db = server_db_path.to_string();
                    move |player| load_deck_by_name(&db, "Sceptile", player)
                }),
            },
            DeckConfig {
                name: "Overzealous-v0".to_string(),
                load_fn: Box::new({
                    let db = server_db_path.to_string();
                    move |player| load_deck_by_name(&db, "Overzealous", player)
                }),
            },
        ]
    } else {
        vec![
            DeckConfig {
                name: "Green Cyclone".to_string(),
                load_fn: Box::new(|player| load_theme_deck("data/theme/green_cyclone.json", player)),
            },
            DeckConfig {
                name: "Sceptile ex Delta".to_string(),
                load_fn: Box::new({
                    let db = server_db_path.to_string();
                    move |player| load_deck_by_name(&db, "Sceptile", player)
                }),
            },
            DeckConfig {
                name: "Overzealous-v0".to_string(),
                load_fn: Box::new({
                    let db = server_db_path.to_string();
                    move |player| load_deck_by_name(&db, "Overzealous", player)
                }),
            },
        ]
    };

    // Verify all decks can be loaded
    println!("\nLoading decks...");
    for deck in &decks {
        match (deck.load_fn)(PlayerId::P1) {
            Some(cards) => println!("  ✓ {}: {} cards", deck.name, cards.len()),
            None => {
                eprintln!("  ✗ {}: Failed to load", deck.name);
                return;
            }
        }
    }

    if bench_matchups {
        let ai_order = [
            AiType::RandomAi,
            AiType::RandomAiV2,
            AiType::RandomAiV3,
            AiType::RandomAiV4,
        ];
        let current_index = ai_order
            .iter()
            .position(|ai| *ai == p1_ai_type)
            .unwrap_or(0);
        let prev_ai_types = &ai_order[..current_index];

        if prev_ai_types.is_empty() {
            println!("\nNo previous AIs to benchmark for {}.", ai_name(p1_ai_type));
            return;
        }

        println!(
            "\nBenchmarking {} vs previous AIs on public decks",
            ai_name(p1_ai_type)
        );

        let mut overall_stats = MatchStats::default();
        let mut opponent_summaries: Vec<(AiType, MatchStats)> = Vec::new();

        for (opp_idx, opponent_ai) in prev_ai_types.iter().enumerate() {
            println!("\nOpponent: {}", ai_name(*opponent_ai));
            let mut opponent_stats = MatchStats::default();

            for (deck_idx, deck) in decks.iter().enumerate() {
                println!(
                    "  {} vs {} on {} ({} matches each direction)",
                    ai_name(p1_ai_type),
                    ai_name(*opponent_ai),
                    deck.name,
                    num_matches
                );

                let seed_base = (deck_idx as u64) * 100_000 + (opp_idx as u64) * 10_000;
                let stats = run_blended_matchup(
                    deck,
                    deck,
                    p1_ai_type,
                    *opponent_ai,
                    num_matches,
                    seed_base,
                    &card_meta,
                    threads,
                    threads <= 1,
                );
                if threads <= 1 {
                    println!();
                }

                let p1_rate = if stats.total > 0 {
                    (stats.p1_wins as f64 / stats.total as f64) * 100.0
                } else {
                    0.0
                };
                let p2_wins = stats.total - stats.p1_wins;
                let p2_rate = if stats.total > 0 { 100.0 - p1_rate } else { 0.0 };

                println!(
                    "    {}: {} wins / {} matches ({:.1}%)",
                    ai_name(p1_ai_type),
                    stats.p1_wins,
                    stats.total,
                    p1_rate
                );
                println!(
                    "    {}: {} wins / {} matches ({:.1}%)",
                    ai_name(*opponent_ai),
                    p2_wins,
                    stats.total,
                    p2_rate
                );

                opponent_stats.merge(stats);
            }

            let opponent_rate = if opponent_stats.total > 0 {
                (opponent_stats.p1_wins as f64 / opponent_stats.total as f64) * 100.0
            } else {
                0.0
            };
            println!(
                "  Aggregate vs {}: {} wins / {} matches ({:.1}%)",
                ai_name(*opponent_ai),
                opponent_stats.p1_wins,
                opponent_stats.total,
                opponent_rate
            );

            opponent_summaries.push((*opponent_ai, opponent_stats.clone()));
            overall_stats.merge(opponent_stats);
        }

        let overall_rate = if overall_stats.total > 0 {
            (overall_stats.p1_wins as f64 / overall_stats.total as f64) * 100.0
        } else {
            0.0
        };
        println!(
            "\nOverall aggregate for {}: {} wins / {} matches ({:.1}%)",
            ai_name(p1_ai_type),
            overall_stats.p1_wins,
            overall_stats.total,
            overall_rate
        );

        if !opponent_summaries.is_empty() {
            println!("\nSummary vs previous AIs:");
            println!("{:18} {:>8} {:>9} {:>8}", "Opponent", "Wins", "Matches", "Win%");
            for (opponent, stats) in &opponent_summaries {
                let rate = if stats.total > 0 {
                    (stats.p1_wins as f64 / stats.total as f64) * 100.0
                } else {
                    0.0
                };
                println!(
                    "{:18} {:>8} {:>9} {:>7.1}%",
                    ai_name(*opponent),
                    stats.p1_wins,
                    stats.total,
                    rate
                );
            }
        }

        print_game_stats(&overall_stats);
        return;
    }

    // Run simulations
    println!("\nRunning simulations...");
    let mut results: HashMap<(usize, usize), (usize, usize)> = HashMap::new(); // (deck1_idx, deck2_idx) -> (wins, total)
    let mut game_durations: Vec<f64> = Vec::new(); // Game durations in seconds
    let mut game_turns: Vec<usize> = Vec::new(); // Number of turns per game

    for (i, deck1) in decks.iter().enumerate() {
        for (j, deck2) in decks.iter().enumerate() {
            if (single_deck_matchup || diagonal_matchups) && i != j {
                continue; // Only run same deck vs same deck
            }
            
            if blend_matchups {
                // Run both matchups and blend results
                println!("\n{} vs {} (blended: {} matches each direction, {} total)", 
                    deck1.name, deck2.name, num_matches, num_matches * 2);
                
                let mut p1_wins = 0; // Wins for p1_ai_type
                let mut total = 0;
                
                let stderr_guard = StderrGuard::new();
                
                // Scenario 1: p1_ai_type as P1, p2_ai_type as P2
                for match_num in 0..num_matches {
                    let seed = (i * 1000 + j * 100 + match_num) as u64;
                    
                    let deck1_cards = (deck1.load_fn)(PlayerId::P1).unwrap();
                    let deck2_cards = (deck2.load_fn)(PlayerId::P2).unwrap();

                    let game = GameState::new_with_card_meta(
                        deck1_cards,
                        deck2_cards,
                        seed,
                        RulesetConfig::default(),
                        card_meta.clone(),
                    );

                    let mut ai1_box: Box<dyn AiController> = match p1_ai_type {
                        AiType::RandomAi => Box::new(RandomAi::new(seed)),
                        AiType::RandomAiV2 => Box::new(RandomAiV2::new(seed)),
                        AiType::RandomAiV3 => Box::new(RandomAiV3::new(seed)),
                        AiType::RandomAiV4 => Box::new(RandomAiV4::new(seed)),
                    };
                    let mut ai2_box: Box<dyn AiController> = match p2_ai_type {
                        AiType::RandomAi => Box::new(RandomAi::new(seed.wrapping_add(9001))),
                        AiType::RandomAiV2 => Box::new(RandomAiV2::new(seed.wrapping_add(9001))),
                        AiType::RandomAiV3 => Box::new(RandomAiV3::new(seed.wrapping_add(9001))),
                        AiType::RandomAiV4 => Box::new(RandomAiV4::new(seed.wrapping_add(9001))),
                    };

                    let start_time = std::time::Instant::now();
                    if let Some((winner, final_game)) = run_match_loop_with_stats(
                        game, 
                        Some(ai1_box.as_mut()),
                        Some(ai2_box.as_mut()),
                        5_000
                    ) {
                        let duration = start_time.elapsed().as_secs_f64();
                        let turn_count = final_game.turn.number as usize;
                        
                        total += 1;
                        if winner == PlayerId::P1 {
                            p1_wins += 1;
                        }
                        
                        game_durations.push(duration);
                        game_turns.push(turn_count);
                    }
                    
                    print!(".");
                    io::stdout().flush().unwrap();
                }
                
                // Scenario 2: p2_ai_type as P1, p1_ai_type as P2 (swapped)
                for match_num in 0..num_matches {
                    let seed = (i * 1000 + j * 100 + match_num + 50000) as u64; // Different seed range
                    
                    let deck1_cards = (deck1.load_fn)(PlayerId::P1).unwrap();
                    let deck2_cards = (deck2.load_fn)(PlayerId::P2).unwrap();

                    let game = GameState::new_with_card_meta(
                        deck1_cards,
                        deck2_cards,
                        seed,
                        RulesetConfig::default(),
                        card_meta.clone(),
                    );

                    // Swapped: p2_ai_type as P1, p1_ai_type as P2
                    let mut ai1_box: Box<dyn AiController> = match p2_ai_type {
                        AiType::RandomAi => Box::new(RandomAi::new(seed)),
                        AiType::RandomAiV2 => Box::new(RandomAiV2::new(seed)),
                        AiType::RandomAiV3 => Box::new(RandomAiV3::new(seed)),
                        AiType::RandomAiV4 => Box::new(RandomAiV4::new(seed)),
                    };
                    let mut ai2_box: Box<dyn AiController> = match p1_ai_type {
                        AiType::RandomAi => Box::new(RandomAi::new(seed.wrapping_add(9001))),
                        AiType::RandomAiV2 => Box::new(RandomAiV2::new(seed.wrapping_add(9001))),
                        AiType::RandomAiV3 => Box::new(RandomAiV3::new(seed.wrapping_add(9001))),
                        AiType::RandomAiV4 => Box::new(RandomAiV4::new(seed.wrapping_add(9001))),
                    };

                    let start_time = std::time::Instant::now();
                    if let Some((winner, final_game)) = run_match_loop_with_stats(
                        game, 
                        Some(ai1_box.as_mut()),
                        Some(ai2_box.as_mut()),
                        5_000
                    ) {
                        let duration = start_time.elapsed().as_secs_f64();
                        let turn_count = final_game.turn.number as usize;
                        
                        total += 1;
                        // In this scenario, if P1 wins, that's p2_ai_type winning, so p1_ai_type loses
                        if winner == PlayerId::P2 {
                            p1_wins += 1;
                        }
                        
                        game_durations.push(duration);
                        game_turns.push(turn_count);
                    }
                    
                    print!(".");
                    io::stdout().flush().unwrap();
                }
                
                drop(stderr_guard);
                println!();
                
                let p1_ai_name = match p1_ai_type {
                    AiType::RandomAi => "RandomAi",
                    AiType::RandomAiV2 => "RandomAiV2",
                    AiType::RandomAiV3 => "RandomAiV3",
                    AiType::RandomAiV4 => "RandomAiV4",
                };
                let p2_ai_name = match p2_ai_type {
                    AiType::RandomAi => "RandomAi",
                    AiType::RandomAiV2 => "RandomAiV2",
                    AiType::RandomAiV3 => "RandomAiV3",
                    AiType::RandomAiV4 => "RandomAiV4",
                };
                let p1_win_rate = if total > 0 { (p1_wins as f64 / total as f64) * 100.0 } else { 0.0 };
                let p2_wins = total - p1_wins;
                let p2_win_rate = if total > 0 { (p2_wins as f64 / total as f64) * 100.0 } else { 0.0 };
                
                println!("\n  Blended Result (accounting for first/second player):");
                println!("    {}: {} wins / {} matches ({:.1}%)", p1_ai_name, p1_wins, total, p1_win_rate);
                println!("    {}: {} wins / {} matches ({:.1}%)", p2_ai_name, p2_wins, total, p2_win_rate);
                
                results.insert((i, j), (p1_wins, total));
            } else {
                // Original behavior: single matchup
                println!("\n{} vs {} ({} matches)", deck1.name, deck2.name, num_matches);

                let mut wins = 0;
                let mut total = 0;

                let stderr_guard = StderrGuard::new();
                
                for match_num in 0..num_matches {
                    let seed = (i * 1000 + j * 100 + match_num) as u64;
                    
                    let deck1_cards = (deck1.load_fn)(PlayerId::P1).unwrap();
                    let deck2_cards = (deck2.load_fn)(PlayerId::P2).unwrap();

                    let game = GameState::new_with_card_meta(
                        deck1_cards,
                        deck2_cards,
                        seed,
                        RulesetConfig::default(),
                        card_meta.clone(),
                    );

                    let mut ai1_box: Box<dyn AiController> = match p1_ai_type {
                        AiType::RandomAi => Box::new(RandomAi::new(seed)),
                        AiType::RandomAiV2 => Box::new(RandomAiV2::new(seed)),
                        AiType::RandomAiV3 => Box::new(RandomAiV3::new(seed)),
                        AiType::RandomAiV4 => Box::new(RandomAiV4::new(seed)),
                    };
                    let mut ai2_box: Box<dyn AiController> = match p2_ai_type {
                        AiType::RandomAi => Box::new(RandomAi::new(seed.wrapping_add(9001))),
                        AiType::RandomAiV2 => Box::new(RandomAiV2::new(seed.wrapping_add(9001))),
                        AiType::RandomAiV3 => Box::new(RandomAiV3::new(seed.wrapping_add(9001))),
                        AiType::RandomAiV4 => Box::new(RandomAiV4::new(seed.wrapping_add(9001))),
                    };

                    let start_time = std::time::Instant::now();
                    if let Some((winner, final_game)) = run_match_loop_with_stats(
                        game, 
                        Some(ai1_box.as_mut()),
                        Some(ai2_box.as_mut()),
                        5_000
                    ) {
                        let duration = start_time.elapsed().as_secs_f64();
                        let turn_count = final_game.turn.number as usize;
                        
                        total += 1;
                        if winner == PlayerId::P1 {
                            wins += 1;
                        }
                        
                        game_durations.push(duration);
                        game_turns.push(turn_count);
                    }
                    
                    print!(".");
                    io::stdout().flush().unwrap();
                }
                drop(stderr_guard);
                println!();

                println!("\n  Result: {} wins / {} matches ({:.1}%)", 
                    wins, total, 
                    if total > 0 { (wins as f64 / total as f64) * 100.0 } else { 0.0 }
                );

                results.insert((i, j), (wins, total));
            }
        }
    }

    // Print results matrix
    println!("\n\nResults Matrix");
    println!("==============");
    println!("\nRows = Player 1, Columns = Player 2");
    println!("Values = Win rate for Player 1\n");

    // Header
    print!("{:20}", "");
    for deck in &decks {
        print!("{:20}", deck.name);
    }
    println!();

    // Rows
    for (i, deck1) in decks.iter().enumerate() {
        print!("{:20}", deck1.name);
        for (j, _deck2) in decks.iter().enumerate() {
            if let Some((wins, total)) = results.get(&(i, j)) {
                if *total > 0 {
                    let win_rate = (*wins as f64 / *total as f64) * 100.0;
                    print!("{:19.1}%", win_rate);
                } else {
                    print!("{:20}", "N/A");
                }
            } else {
                print!("{:20}", "N/A");
            }
        }
        println!();
    }

    // Summary statistics
    println!("\n\nOverall Win Rates (across all matchups):");
    println!("==========================================");
    
    if blend_matchups {
        // When comparing AIs, show per-deck breakdown then aggregate
        let p1_ai_name = match p1_ai_type { 
            AiType::RandomAi => "RandomAi", 
            AiType::RandomAiV2 => "RandomAiV2", 
            AiType::RandomAiV3 => "RandomAiV3",
            AiType::RandomAiV4 => "RandomAiV4",
        };
        let p2_ai_name = match p2_ai_type { 
            AiType::RandomAi => "RandomAi", 
            AiType::RandomAiV2 => "RandomAiV2", 
            AiType::RandomAiV3 => "RandomAiV3",
            AiType::RandomAiV4 => "RandomAiV4",
        };
        
        let mut p1_total_wins = 0;
        let mut p1_total_matches = 0;
        let mut p2_total_wins = 0;
        let mut p2_total_matches = 0;
        
        // Show per-deck breakdown
        println!("Per-Deck Breakdown:");
        for (i, deck) in decks.iter().enumerate() {
            if let Some((wins, matches)) = results.get(&(i, i)) {
                let p1_wins = *wins;
                let p2_wins = matches - wins;
                let p1_rate = if *matches > 0 {
                    (p1_wins as f64 / *matches as f64) * 100.0
                } else {
                    0.0
                };
                let p2_rate = if *matches > 0 {
                    (p2_wins as f64 / *matches as f64) * 100.0
                } else {
                    0.0
                };
                
                println!("  {}:", deck.name);
                println!("    {:20} {:4}/{:4} ({:.1}%)", p1_ai_name, p1_wins, matches, p1_rate);
                println!("    {:20} {:4}/{:4} ({:.1}%)", p2_ai_name, p2_wins, matches, p2_rate);
                
                p1_total_wins += p1_wins;
                p1_total_matches += matches;
                p2_total_wins += p2_wins;
                p2_total_matches += matches;
            }
        }
        
        // Show aggregate across all decks
        println!("\nAggregate (across all decks):");
        let p1_rate = if p1_total_matches > 0 {
            (p1_total_wins as f64 / p1_total_matches as f64) * 100.0
        } else {
            0.0
        };
        let p2_rate = if p2_total_matches > 0 {
            (p2_total_wins as f64 / p2_total_matches as f64) * 100.0
        } else {
            0.0
        };
        
        println!("  {:20} {:4}/{:4} ({:.1}%)", p1_ai_name, p1_total_wins, p1_total_matches, p1_rate);
        println!("  {:20} {:4}/{:4} ({:.1}%)", p2_ai_name, p2_total_wins, p2_total_matches, p2_rate);
    } else {
        // When comparing decks, aggregate by deck
        let mut deck_wins: Vec<(String, usize, usize)> = decks
            .iter()
            .enumerate()
            .map(|(i, deck)| {
                let mut total_wins = 0;
                let mut total_matches = 0;
                for j in 0..decks.len() {
                    if let Some((wins, matches)) = results.get(&(i, j)) {
                        total_wins += wins;
                        total_matches += matches;
                    }
                }
                (deck.name.clone(), total_wins, total_matches)
            })
            .collect();

        deck_wins.sort_by(|a, b| {
            let rate_a = if a.2 > 0 { a.1 as f64 / a.2 as f64 } else { 0.0 };
            let rate_b = if b.2 > 0 { b.1 as f64 / b.2 as f64 } else { 0.0 };
            rate_b.partial_cmp(&rate_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        for (name, wins, total) in deck_wins {
            let win_rate = if total > 0 {
                (wins as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            println!("  {:20} {:4}/{:4} ({:.1}%)", name, wins, total, win_rate);
        }
    }

    // Print game statistics
    if !game_durations.is_empty() && !game_turns.is_empty() {
        println!("\n\nGame Statistics:");
        println!("================");
        
        // Sort for percentile calculations
        game_durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        game_turns.sort();
        
        let n = game_durations.len();
        let median_idx = n / 2;
        let p90_idx = (n as f64 * 0.9) as usize;
        
        println!("\nGame Duration (seconds):");
        println!("  Median:  {:.3}s", game_durations[median_idx]);
        println!("  P90:     {:.3}s", game_durations[p90_idx.min(n - 1)]);
        println!("  Max:     {:.3}s", game_durations[n - 1]);
        
        println!("\nNumber of Turns:");
        println!("  Median:  {}", game_turns[median_idx]);
        println!("  P90:     {}", game_turns[p90_idx.min(n - 1)]);
        println!("  Max:     {}", game_turns[n - 1]);
        println!("  Total games: {}", n);
    }
}
