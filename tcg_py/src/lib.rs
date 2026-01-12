//! Python bindings for Pokemon TCG game engine.
//!
//! This crate provides PyO3 bindings to run Pokemon TCG games
//! between an LLM-based agent and the AI v4 opponent.

use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;

mod ai;

use ai::{AiController, RandomAiV4};
use ai::react::{render_game_view, render_game_view_compact, parse_response};

use tcg_core::{GameState, PlayerId, StepResult, GameView};
use tcg_rules_ex::RulesetConfig;

/// Result of a completed game.
#[pyclass]
#[derive(Clone)]
pub struct PyGameResult {
    #[pyo3(get)]
    pub winner: Option<String>,
    #[pyo3(get)]
    pub turns: u32,
    #[pyo3(get)]
    pub steps: u32,
    #[pyo3(get)]
    pub p1_prizes_remaining: usize,
    #[pyo3(get)]
    pub p2_prizes_remaining: usize,
    #[pyo3(get)]
    pub end_reason: String,
    #[pyo3(get)]
    pub history: Vec<PyTurnSummary>,
}

#[pymethods]
impl PyGameResult {
    fn __repr__(&self) -> String {
        format!(
            "GameResult(winner={:?}, turns={}, prizes=({}, {}), reason={})",
            self.winner, self.turns, self.p1_prizes_remaining, self.p2_prizes_remaining, self.end_reason
        )
    }
}

/// Summary of a turn.
#[pyclass]
#[derive(Clone)]
pub struct PyTurnSummary {
    #[pyo3(get)]
    pub turn: u32,
    #[pyo3(get)]
    pub player: String,
    #[pyo3(get)]
    pub actions: Vec<String>,
    #[pyo3(get)]
    pub state_compact: String,
}

/// Game observation for the LLM agent.
#[pyclass]
#[derive(Clone)]
pub struct PyObservation {
    #[pyo3(get)]
    pub game_state: String,
    #[pyo3(get)]
    pub compact: String,
    #[pyo3(get)]
    pub phase: String,
    #[pyo3(get)]
    pub current_player: String,
    #[pyo3(get)]
    pub has_prompt: bool,
    #[pyo3(get)]
    pub prompt_type: Option<String>,
    #[pyo3(get)]
    pub available_actions: Vec<String>,
    #[pyo3(get)]
    pub my_prizes: usize,
    #[pyo3(get)]
    pub opp_prizes: usize,
}

fn create_observation(view: &GameView) -> PyObservation {
    let hints = &view.action_hints;
    let mut available_actions = Vec::new();

    if !hints.playable_basic_ids.is_empty() {
        available_actions.push(format!("PlayBasic ({})", hints.playable_basic_ids.len()));
    }
    if !hints.playable_energy_ids.is_empty() {
        available_actions.push(format!("AttachEnergy ({})", hints.playable_energy_ids.len()));
    }
    if !hints.playable_evolution_ids.is_empty() {
        available_actions.push(format!("Evolve ({})", hints.playable_evolution_ids.len()));
    }
    if !hints.playable_trainer_ids.is_empty() {
        available_actions.push(format!("PlayTrainer ({})", hints.playable_trainer_ids.len()));
    }
    if hints.can_declare_attack {
        available_actions.push(format!("Attack ({})", hints.usable_attacks.len()));
    }
    if hints.can_end_turn {
        available_actions.push("EndTurn".to_string());
    }

    PyObservation {
        game_state: render_game_view(view),
        compact: render_game_view_compact(view),
        phase: format!("{:?}", view.phase),
        current_player: format!("{:?}", view.current_player),
        has_prompt: view.pending_prompt.is_some(),
        prompt_type: view.pending_prompt.as_ref().map(|p| format!("{:?}", p).split('{').next().unwrap_or("Unknown").trim().to_string()),
        available_actions,
        my_prizes: view.my_prizes_count,
        opp_prizes: view.opponent_prizes_count,
    }
}

/// A Pokemon TCG game that can be played step by step.
#[pyclass]
pub struct PtcgGame {
    game: GameState,
    v4_ai: RandomAiV4,
    steps: u32,
    turns: u32,
    max_steps: u32,
    history: Vec<PyTurnSummary>,
    current_turn_actions: Vec<String>,
    game_over: bool,
    winner: Option<PlayerId>,
}

#[pymethods]
impl PtcgGame {
    /// Create a new game with the given decks and seeds.
    #[new]
    #[pyo3(signature = (p1_deck, p2_deck, game_seed=42, ai_seed=123, max_steps=1000))]
    fn new(
        p1_deck: Vec<String>,
        p2_deck: Vec<String>,
        game_seed: u64,
        ai_seed: u64,
        max_steps: u32,
    ) -> PyResult<Self> {
        // Create decks from card IDs
        let deck1 = p1_deck.iter()
            .enumerate()
            .map(|(_i, def_id)| tcg_core::CardInstance::new(
                tcg_core::CardDefId::new(def_id.clone()),
                PlayerId::P1,
            ))
            .collect();

        let deck2 = p2_deck.iter()
            .enumerate()
            .map(|(_i, def_id)| tcg_core::CardInstance::new(
                tcg_core::CardDefId::new(def_id.clone()),
                PlayerId::P2,
            ))
            .collect();

        let game = GameState::new(deck1, deck2, game_seed, RulesetConfig::default());
        let v4_ai = RandomAiV4::new(ai_seed);

        Ok(Self {
            game,
            v4_ai,
            steps: 0,
            turns: 0,
            max_steps,
            history: Vec::new(),
            current_turn_actions: Vec::new(),
            game_over: false,
            winner: None,
        })
    }

    /// Get the current observation for player 1 (the LLM agent).
    fn get_observation(&self) -> PyObservation {
        let view = self.game.view_for_player(PlayerId::P1);
        create_observation(&view)
    }

    /// Check if it's the LLM agent's turn (P1).
    fn is_agent_turn(&self) -> bool {
        self.game.turn.player == PlayerId::P1
    }

    /// Check if the game is over.
    fn is_game_over(&self) -> bool {
        self.game_over
    }

    /// Get the winner if game is over.
    fn get_winner(&self) -> Option<String> {
        self.winner.map(|w| format!("{:?}", w))
    }

    /// Step the game forward, handling AI turns automatically.
    /// Returns the new observation for P1.
    fn step(&mut self) -> PyResult<PyObservation> {
        if self.game_over {
            return Ok(self.get_observation());
        }

        // Run one game step
        let result = self.game.step();
        self.steps += 1;

        match result {
            StepResult::GameOver { winner } => {
                self.game_over = true;
                self.winner = Some(winner);
            }
            StepResult::Prompt { prompt, for_player } => {
                if for_player == PlayerId::P2 {
                    // AI handles prompt
                    let view = self.game.view_for_player(PlayerId::P2);
                    let actions = self.v4_ai.propose_prompt_response(&view, &prompt);
                    for action in actions {
                        let _ = self.game.apply_action(PlayerId::P2, action);
                    }
                }
                // If P1 prompt, return and let caller handle it
            }
            StepResult::Continue => {
                // Check for AI free actions
                if self.game.turn.player == PlayerId::P2 {
                    let view = self.game.view_for_player(PlayerId::P2);
                    if matches!(view.phase, tcg_rules_ex::Phase::Main | tcg_rules_ex::Phase::Attack) {
                        let actions = self.v4_ai.propose_free_actions(&view);
                        if let Some(action) = actions.into_iter().next() {
                            let _ = self.game.apply_action(PlayerId::P2, action);
                        }
                    }
                }
            }
            StepResult::Event { .. } => {}
        }

        // Check max steps
        if self.steps >= self.max_steps {
            self.game_over = true;
        }

        Ok(self.get_observation())
    }

    /// Submit an action from the LLM agent (P1).
    /// The action should be a JSON string like: {"action": "EndTurn"}
    fn submit_action(&mut self, action_json: &str) -> PyResult<bool> {
        if self.game_over {
            return Err(PyRuntimeError::new_err("Game is already over"));
        }

        let view = self.game.view_for_player(PlayerId::P1);

        // Parse the action
        let parsed = match parse_response(action_json, &view) {
            Ok(p) => p,
            Err(e) => {
                return Err(PyRuntimeError::new_err(format!(
                    "Failed to parse action: {:?}",
                    e
                )));
            }
        };
        let action = parsed.action;

        // Apply the action
        match self.game.apply_action(PlayerId::P1, action) {
            Ok(_) => {
                self.current_turn_actions.push(action_json.to_string());
                Ok(true)
            }
            Err(e) => Err(PyRuntimeError::new_err(format!("Action failed: {:?}", e))),
        }
    }

    /// Run the game until it needs P1 input or ends.
    /// Returns observation when P1 action is needed, or final observation.
    fn run_until_agent_turn(&mut self) -> PyResult<PyObservation> {
        while !self.game_over && !self.is_agent_turn() {
            self.step()?;
        }

        // Also step through any non-prompt Continue states
        while !self.game_over && self.is_agent_turn() {
            let view = self.game.view_for_player(PlayerId::P1);
            if view.pending_prompt.is_some() ||
               matches!(view.phase, tcg_rules_ex::Phase::Main | tcg_rules_ex::Phase::Attack) {
                break;
            }
            self.step()?;
        }

        Ok(self.get_observation())
    }

    /// Get the final game result.
    fn get_result(&self) -> PyGameResult {
        let end_reason = if self.winner.is_some() {
            "GameOver"
        } else if self.steps >= self.max_steps {
            "MaxSteps"
        } else {
            "InProgress"
        };

        PyGameResult {
            winner: self.winner.map(|w| format!("{:?}", w)),
            turns: self.turns,
            steps: self.steps,
            p1_prizes_remaining: self.game.view_for_player(PlayerId::P1).my_prizes_count,
            p2_prizes_remaining: self.game.view_for_player(PlayerId::P2).my_prizes_count,
            end_reason: end_reason.to_string(),
            history: self.history.clone(),
        }
    }
}

/// Run a complete game with a Python LLM callback.
///
/// Args:
///     p1_deck: List of card definition IDs for player 1's deck
///     p2_deck: List of card definition IDs for player 2's deck
///     llm_callback: Python callable that takes (observation: str) and returns action JSON
///     game_seed: Random seed for the game
///     ai_seed: Random seed for AI v4
///     max_steps: Maximum game steps before stopping
///
/// Returns:
///     GameResult with winner, turns, etc.
#[pyfunction]
#[pyo3(signature = (p1_deck, p2_deck, llm_callback, game_seed=42, ai_seed=123, max_steps=1000))]
fn run_game_vs_ai(
    py: Python<'_>,
    p1_deck: Vec<String>,
    p2_deck: Vec<String>,
    llm_callback: PyObject,
    game_seed: u64,
    ai_seed: u64,
    max_steps: u32,
) -> PyResult<PyGameResult> {
    let mut game = PtcgGame::new(p1_deck, p2_deck, game_seed, ai_seed, max_steps)?;

    while !game.is_game_over() {
        // Run until agent needs to act
        let obs = game.run_until_agent_turn()?;

        if game.is_game_over() {
            break;
        }

        // Call LLM callback with observation
        let action_json: String = llm_callback.call1(py, (obs.game_state.clone(),))?.extract(py)?;

        // Submit action
        match game.submit_action(&action_json) {
            Ok(_) => {}
            Err(e) => {
                // On error, try EndTurn as fallback
                let _ = game.submit_action(r#"{"action": "EndTurn"}"#);
            }
        }

        // Step to process the action
        game.step()?;
    }

    Ok(game.get_result())
}

/// Python module for Pokemon TCG game engine.
#[pymodule]
fn tcg_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PtcgGame>()?;
    m.add_class::<PyGameResult>()?;
    m.add_class::<PyTurnSummary>()?;
    m.add_class::<PyObservation>()?;
    m.add_function(wrap_pyfunction!(run_game_vs_ai, m)?)?;
    Ok(())
}
