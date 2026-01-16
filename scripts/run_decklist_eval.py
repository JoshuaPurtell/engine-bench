#!/usr/bin/env python3
"""Decklist evaluation for EngineBench.

Runs a single agent session to implement all cards in a decklist,
then evaluates each card individually.

Usage:
    python scripts/run_decklist_eval.py --model openai/gpt-5-nano --decklist df-delta-allstars
"""

import argparse
import json
import shutil
import subprocess
import tempfile
import time
from datetime import datetime
from pathlib import Path

BASE_DIR = Path(__file__).parent.parent
DATA_DIR = BASE_DIR / "data"
OVERZEALOUS_REPO = Path.home() / "Documents" / "GitHub" / "overzealous"


def load_decklist(decklist_id: str) -> dict:
    """Load decklist specification."""
    decklist_path = DATA_DIR / "instances" / "decklist" / f"{decklist_id}.json"
    if not decklist_path.exists():
        raise ValueError(f"Decklist not found: {decklist_id}")
    return json.loads(decklist_path.read_text())


def load_card_instance(instance_id: str) -> dict:
    """Load a single card instance."""
    instance_path = DATA_DIR / "instances" / "single" / f"{instance_id}.json"
    if not instance_path.exists():
        raise ValueError(f"Card instance not found: {instance_id}")
    return json.loads(instance_path.read_text())


def get_all_cards(decklist: dict) -> list:
    """Get all cards (Pokemon and trainers) from decklist."""
    cards = list(decklist.get("cards", []))
    cards.extend(decklist.get("trainers", []))
    return cards


def setup_sandbox(decklist: dict, work_dir: Path) -> Path:
    """Set up sandbox with stubs for all cards in decklist."""
    sandbox_dir = work_dir / "overzealous"

    print(f"  Copying repo to {sandbox_dir}...")
    shutil.copytree(
        OVERZEALOUS_REPO,
        sandbox_dir,
        symlinks=True,
        ignore=shutil.ignore_patterns(".git", "target", "*.pyc", "__pycache__"),
    )

    # Replace each card with its stub (both Pokemon and trainers)
    all_cards = get_all_cards(decklist)
    for card in all_cards:
        instance_id = card["instance_id"]
        stub_file = BASE_DIR / "gold" / "stubs" / f"{instance_id.replace('-', '_')}.rs"

        if stub_file.exists():
            card_path = sandbox_dir / "tcg_expansions" / "src" / "df" / "cards" / f"{instance_id.replace('-', '_')}.rs"
            print(f"  Installing stub for {card['name']}...")
            card_path.write_text(stub_file.read_text())
        else:
            print(f"  WARNING: No stub found for {instance_id}")

    return sandbox_dir


def build_decklist_prompt(decklist: dict) -> str:
    """Build prompt for implementing all cards in a decklist."""
    # Pokemon cards
    pokemon_info = []
    for card in decklist.get("cards", []):
        instance_id = card["instance_id"]
        try:
            card_instance = load_card_instance(instance_id)
            card_data = card_instance["cards"][0]
            card_file = card_instance["card_file"]

            # Build card spec
            abilities = card_data.get("abilities", [])
            attacks = card_data.get("attacks", [])

            ability_text = ""
            for ab in abilities:
                ability_text += f"\n  - {ab['name']} ({ab['type']}): {ab['text']}"

            attack_text = ""
            for atk in attacks:
                if atk.get("text"):
                    attack_text += f"\n  - {atk['name']}: {atk.get('damage', 0)} damage. {atk['text']}"

            pokemon_info.append(f"""
### {card_data['name']} (#{card_data['number']})
File: `{card_file}`
Role in deck: {card['role']}
{f"Abilities:{ability_text}" if ability_text else ""}
{f"Attacks:{attack_text}" if attack_text else ""}
""")
        except Exception as e:
            print(f"  Error loading {instance_id}: {e}")

    # Trainer cards
    trainer_info = []
    for card in decklist.get("trainers", []):
        instance_id = card["instance_id"]
        try:
            card_instance = load_card_instance(instance_id)
            card_data = card_instance["cards"][0]
            card_file = card_instance["card_file"]

            trainer_info.append(f"""
### {card_data['name']} (#{card_data['number']})
Type: {card_data.get('trainer_kind', 'Trainer').title()}
File: `{card_file}`
Role in deck: {card['role']}
Effect: {card_data.get('effect', '')}
""")
        except Exception as e:
            print(f"  Error loading {instance_id}: {e}")

    pokemon_section = "\n".join(pokemon_info)
    trainer_section = "\n".join(trainer_info)

    total_implementations = decklist.get("unique_implementations", len(pokemon_info) + len(trainer_info))

    return f'''You are implementing a Pokemon TCG decklist for the Dragon Frontiers expansion in Rust.

## Decklist: {decklist["name"]}
{decklist.get("description", "")}

## Task
Implement ALL {total_implementations} unique cards in this decklist. For each card:
1. Read the stub file
2. Replace TODO implementations with working code
3. Ensure it compiles
4. Ensure tests pass

## Pokemon Cards to Implement
{pokemon_section}

## Trainer Cards to Implement
{trainer_section if trainer_section else "(No trainer cards to implement)"}

## CRITICAL Instructions
1. Implement EACH card one by one
2. For each card:
   - READ the stub file at the path shown
   - READ similar implementations in tcg_expansions/src/cg/cards/ for patterns
   - EDIT the file to replace TODO stubs with working code
3. After implementing all cards, run `cargo check --package tcg_expansions` to verify
4. Run `cargo test --package tcg_expansions` to verify all tests pass

## Reference Patterns
Look at existing card implementations in `tcg_expansions/src/cg/` for:
- How to implement Poke-Powers (execute_* functions)
- How to implement Poke-Bodies (check/bonus functions)
- How to implement attack modifiers (*_bonus functions)
- How to search decks, move cards, apply conditions
- How to implement Trainers (execute, can_play functions)

START with the simplest cards first (Basic Pokemon with simple attacks) then work up to complex ones (Stage 2s with powers, trainers with effects).

You MUST actually edit each file and write working code. Do not just describe - implement!
'''


def run_opencode(prompt: str, sandbox_dir: Path, model: str, timeout: int = 1800) -> dict:
    """Run OpenCode on the sandbox."""
    print(f"  Running OpenCode with {model}...")
    print(f"  Timeout: {timeout}s ({timeout/60:.0f} minutes)")

    cmd = [
        "opencode", "run",
        "--model", model,
        "--agent", "build",
        "--format", "json",
        prompt,
    ]

    try:
        proc = subprocess.Popen(
            cmd,
            cwd=str(sandbox_dir),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )

        stdout, stderr = proc.communicate(timeout=timeout)

        return {
            "success": proc.returncode == 0,
            "stdout": stdout,
            "stderr": stderr,
        }
    except subprocess.TimeoutExpired:
        proc.kill()
        stdout, stderr = proc.communicate()
        return {
            "success": False,
            "stdout": stdout or "",
            "stderr": f"Timeout after {timeout} seconds",
        }
    except Exception as e:
        return {
            "success": False,
            "stdout": "",
            "stderr": str(e),
        }


def run_cargo_check(sandbox_dir: Path) -> tuple[bool, str]:
    """Run cargo check."""
    result = subprocess.run(
        ["cargo", "check", "--package", "tcg_expansions"],
        cwd=str(sandbox_dir),
        capture_output=True,
        text=True,
        timeout=300,
    )
    return result.returncode == 0, result.stderr


def evaluate_card(sandbox_dir: Path, instance_id: str) -> dict:
    """Evaluate a single card's implementation."""
    import re

    result = {
        "instance_id": instance_id,
        "compile_pass": False,
        "tests_passed": 0,
        "tests_total": 0,
        "score": 0.0,
        "error": None,
    }

    # Check if eval tests exist
    eval_test_file = BASE_DIR / "gold" / "tests" / f"{instance_id.replace('-', '_')}_eval.rs"
    if not eval_test_file.exists():
        result["error"] = "No eval tests found"
        return result

    # Inject eval tests
    card_file = sandbox_dir / "tcg_expansions" / "src" / "df" / "cards" / f"{instance_id.replace('-', '_')}.rs"
    if not card_file.exists():
        result["error"] = "Card file not found"
        return result

    # Check if already has eval tests (from previous injection)
    current_content = card_file.read_text()
    if "EVALUATION TESTS (injected" not in current_content:
        eval_tests = eval_test_file.read_text()
        eval_module = f'''

// ============================================================================
// EVALUATION TESTS (injected after agent completion)
// ============================================================================

{eval_tests}
'''
        card_file.write_text(current_content + eval_module)

    # Run tests for this specific card
    test_filter = instance_id.replace("-", "_")
    try:
        test_result = subprocess.run(
            ["cargo", "test", "--package", "tcg_expansions", "--", "--test-threads=1", test_filter],
            cwd=str(sandbox_dir),
            capture_output=True,
            text=True,
            timeout=120,
        )

        output = test_result.stdout + test_result.stderr

        # Parse results
        passed = 0
        failed = 0
        for line in output.split("\n"):
            if "test result:" in line:
                match_passed = re.search(r"(\d+) passed", line)
                match_failed = re.search(r"(\d+) failed", line)
                if match_passed:
                    passed += int(match_passed.group(1))
                if match_failed:
                    failed += int(match_failed.group(1))

        total = passed + failed
        result["tests_passed"] = passed
        result["tests_total"] = total
        result["compile_pass"] = test_result.returncode == 0 or total > 0

        if total > 0:
            result["score"] = passed / total
        elif result["compile_pass"]:
            result["score"] = 0.3  # Compiles but no tests ran

    except subprocess.TimeoutExpired:
        result["error"] = "Test timeout"
    except Exception as e:
        result["error"] = str(e)

    return result


def save_decklist_result(decklist_id: str, model: str, card_results: list,
                         total_duration: float, agent_output: str = ""):
    """Save decklist evaluation results."""
    results_dir = BASE_DIR / "results" / "decklist"
    results_dir.mkdir(parents=True, exist_ok=True)

    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    model_safe = model.replace("/", "_").replace(".", "-")
    result_file = results_dir / f"{decklist_id}_{model_safe}_{timestamp}.txt"

    # Calculate summary stats
    total_cards = len(card_results)
    perfect_cards = sum(1 for r in card_results if r["score"] == 1.0)
    partial_cards = sum(1 for r in card_results if 0 < r["score"] < 1.0)
    failed_cards = sum(1 for r in card_results if r["score"] == 0)
    avg_score = sum(r["score"] for r in card_results) / total_cards if total_cards > 0 else 0

    # Format duration
    if total_duration >= 60:
        duration_str = f"{total_duration / 60:.1f} minutes ({total_duration:.1f}s)"
    else:
        duration_str = f"{total_duration:.1f} seconds"

    # Build per-card table
    card_table = ""
    for r in sorted(card_results, key=lambda x: x["instance_id"]):
        status = "PASS" if r["score"] == 1.0 else f"{r['score']:.0%}"
        tests = f"{r['tests_passed']}/{r['tests_total']}"
        error = f" - {r['error']}" if r.get("error") else ""
        card_table += f"  {r['instance_id']:<25} {status:>8} {tests:>10}{error}\n"

    result = f"""EngineBench Decklist Evaluation Result
======================================
Decklist: {decklist_id}
Model: {model}
Timestamp: {timestamp}
Duration: {duration_str}

SUMMARY
=======
Total Cards: {total_cards}
Perfect (100%): {perfect_cards}
Partial: {partial_cards}
Failed (0%): {failed_cards}
Average Score: {avg_score:.1%}

PER-CARD RESULTS
================
{"Instance":<27} {"Score":>8} {"Tests":>10}
{"-"*50}
{card_table}
{"-"*50}

"""

    result_file.write_text(result)
    print(f"\n   Result saved to: {result_file}")
    return result_file


def main():
    parser = argparse.ArgumentParser(description="Run decklist EngineBench evaluation")
    parser.add_argument("--model", type=str, default="openai/gpt-5-nano", help="Model to use")
    parser.add_argument("--decklist", type=str, default="df-delta-allstars", help="Decklist ID")
    parser.add_argument("--timeout", type=int, default=1800, help="Timeout in seconds (default 30 min)")
    args = parser.parse_args()

    print(f"\n{'='*60}")
    print(f"EngineBench Decklist Evaluation")
    print(f"{'='*60}")
    print(f"Model: {args.model}")
    print(f"Decklist: {args.decklist}")
    print(f"Timeout: {args.timeout}s ({args.timeout/60:.0f} minutes)")
    print(f"{'='*60}\n")

    # Load decklist
    print("1. Loading decklist...")
    decklist = load_decklist(args.decklist)
    print(f"   Name: {decklist['name']}")
    all_cards = get_all_cards(decklist)
    pokemon_count = len(decklist.get("cards", []))
    trainer_count = len(decklist.get("trainers", []))
    print(f"   Pokemon: {pokemon_count} unique")
    print(f"   Trainers: {trainer_count} unique")
    print(f"   Total implementations: {len(all_cards)}")
    print("\n   Pokemon cards:")
    for card in decklist.get("cards", []):
        print(f"     - {card['name']}: {card['role']}")
    if decklist.get("trainers"):
        print("\n   Trainer cards:")
        for card in decklist.get("trainers", []):
            print(f"     - {card['name']}: {card['role']}")

    # Setup sandbox
    print("\n2. Setting up sandbox...")
    tmpdir = tempfile.mkdtemp()
    try:
        work_dir = Path(tmpdir)
        sandbox_dir = setup_sandbox(decklist, work_dir)

        # Build prompt
        print("\n3. Building prompt...")
        prompt = build_decklist_prompt(decklist)

        # Run agent
        print("\n4. Running agent...")
        start_time = time.time()
        agent_result = run_opencode(prompt, sandbox_dir, args.model, args.timeout)
        end_time = time.time()
        duration = end_time - start_time

        if duration >= 60:
            print(f"   Agent finished in {duration/60:.1f} minutes")
        else:
            print(f"   Agent finished in {duration:.1f} seconds")

        if not agent_result["success"]:
            print(f"   Agent error: {agent_result['stderr'][:300]}")

        # Check compilation
        print("\n5. Checking compilation...")
        compile_pass, compile_error = run_cargo_check(sandbox_dir)
        if compile_pass:
            print("   Compilation passed")
        else:
            print(f"   Compilation failed (will still evaluate individual cards)")

        # Evaluate each card (Pokemon and trainers)
        print("\n6. Evaluating individual cards...")
        card_results = []
        all_cards = get_all_cards(decklist)
        for card in all_cards:
            instance_id = card["instance_id"]
            print(f"   Testing {card['name']}...", end=" ", flush=True)
            result = evaluate_card(sandbox_dir, instance_id)
            card_results.append(result)

            if result["score"] == 1.0:
                print(f"PASS ({result['tests_passed']}/{result['tests_total']})")
            elif result["error"]:
                print(f"ERROR: {result['error']}")
            else:
                print(f"{result['score']:.0%} ({result['tests_passed']}/{result['tests_total']})")

        # Summary
        total_cards = len(card_results)
        perfect = sum(1 for r in card_results if r["score"] == 1.0)
        avg_score = sum(r["score"] for r in card_results) / total_cards if total_cards else 0

        print(f"\n{'='*60}")
        print("RESULTS")
        print(f"{'='*60}")
        print(f"  Duration: {duration/60:.1f} minutes")
        print(f"  Compile: {'PASS' if compile_pass else 'FAIL'}")
        print(f"  Cards Perfect: {perfect}/{total_cards}")
        print(f"  Average Score: {avg_score:.1%}")
        print(f"\n  Per-Card Breakdown:")
        print(f"  {'Card':<25} {'Score':>8} {'Tests':>10}")
        print(f"  {'-'*45}")
        for r in sorted(card_results, key=lambda x: x["instance_id"]):
            score_str = "PASS" if r["score"] == 1.0 else f"{r['score']:.0%}"
            tests_str = f"{r['tests_passed']}/{r['tests_total']}"
            print(f"  {r['instance_id']:<25} {score_str:>8} {tests_str:>10}")
        print(f"{'='*60}")

        # Save results
        agent_log = agent_result.get("stdout", "") + "\n" + agent_result.get("stderr", "")
        save_decklist_result(args.decklist, args.model, card_results, duration, agent_log)

    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)


if __name__ == "__main__":
    main()
