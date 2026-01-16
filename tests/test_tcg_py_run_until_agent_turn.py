import re
import subprocess
import sys
from pathlib import Path

import pytest
import json


def _first_card_id(game_state: str) -> int:
    match = re.search(r"\(id:(\d+)\)", game_state)
    if not match:
        raise AssertionError("no card id found in game_state")
    return int(match.group(1))


def _ensure_tcg_py():
    try:
        import tcg_py  # noqa: F401
        return tcg_py
    except Exception:
        pass

    tcg_py_dir = Path(__file__).resolve().parents[1] / "tcg_py"
    if not tcg_py_dir.exists():
        pytest.skip("tcg_py source directory not found")

    try:
        subprocess.run(
            [sys.executable, "-m", "maturin", "build", "--release", "-i", sys.executable],
            cwd=str(tcg_py_dir),
            check=True,
        )
        wheels_dir = tcg_py_dir / "target" / "wheels"
        wheels = sorted(wheels_dir.glob("tcg_py-*.whl"), key=lambda p: p.stat().st_mtime, reverse=True)
        if not wheels:
            pytest.skip("tcg_py wheel not found after build")
        subprocess.run(
            [sys.executable, "-m", "pip", "install", "--force-reinstall", str(wheels[0])],
            check=True,
        )
        import tcg_py  # noqa: F401
        return tcg_py
    except Exception as exc:
        pytest.skip(f"tcg_py not available: {exc}")


def _handle_p1_turn(game) -> None:
    obs = game.run_until_agent_turn()
    assert not (obs.current_player == "P2" and obs.phase == "EndOfTurn"), (
        "run_until_agent_turn should not return P2 EndOfTurn"
    )
    assert obs.prompt_json, "prompt_json should be present"
    snapshot = json.loads(obs.prompt_json)
    assert "available_actions" in snapshot
    assert "pending_prompt" in snapshot

    if obs.has_prompt:
        if obs.prompt_type == "ChooseStartingActive":
            card_id = _first_card_id(obs.game_state)
            game.submit_action(f'{{"action": "ChooseActive", "card_id": {card_id}}}')
            game.step()
            return
        if obs.prompt_type == "ChooseBenchBasics":
            game.submit_action('{"action": "ChooseBench", "card_ids": []}')
            game.step()
            return

    if "EndTurn" in obs.available_actions:
        game.submit_action('{"action": "EndTurn"}')
    game.step()


def test_run_until_agent_turn_never_returns_p2_end_of_turn() -> None:
    tcg_py = _ensure_tcg_py()
    deck = ["df-061-ralts"] * 60
    game = tcg_py.PtcgGame(
        p1_deck=deck,
        p2_deck=deck,
        game_seed=42,
        ai_seed=42,
        max_steps=500,
    )

    for _ in range(15):
        if game.is_game_over():
            break
        _handle_p1_turn(game)
