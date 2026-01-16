#!/usr/bin/env python3
"""Quick local evaluation for EngineBench.

Runs OpenCode directly (no Docker) for rapid testing.

Usage:
    # Single instance
    python scripts/run_local_eval.py --model openai/gpt-5-nano --instance df-007-nidoqueen

    # All instances in parallel
    python scripts/run_local_eval.py --model openai/gpt-5-nano --all --workers 4
"""

import argparse
import json
import shutil
import subprocess
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime
from pathlib import Path
from typing import Optional

BASE_DIR = Path(__file__).parent.parent
DATA_DIR = BASE_DIR / "data"
OVERZEALOUS_REPO = Path.home() / "Documents" / "GitHub" / "overzealous"


def get_all_instances() -> list[str]:
    """Get all available instance IDs."""
    instances_dir = DATA_DIR / "instances" / "single"
    instances = []
    for f in instances_dir.glob("*.json"):
        instance_id = f.stem
        # Check if we have eval tests for this instance
        eval_file = BASE_DIR / "gold" / "tests" / f"{instance_id.replace('-', '_')}_eval.rs"
        if eval_file.exists():
            instances.append(instance_id)
    return sorted(instances)


def load_instance(instance_id: str) -> dict:
    """Load instance specification."""
    instance_path = DATA_DIR / "instances" / "single" / f"{instance_id}.json"
    if not instance_path.exists():
        raise ValueError(f"Instance not found: {instance_id}")
    return json.loads(instance_path.read_text())


def build_prompt(instance: dict) -> str:
    """Build the prompt for the coding agent."""
    cards = instance.get("cards", [])
    card_file = instance.get("card_file", "")

    card_specs = "\n\n".join([
        f"### {card['name']}\n```json\n{json.dumps(card, indent=2)}\n```"
        for card in cards
    ])

    tests = instance.get("tests", [])
    def format_test(t):
        desc = t.get('description')
        if desc:
            return f"- {t['name']}: {desc}"
        return f"- {t['name']}: expected={t.get('expected', '?')}"
    test_descriptions = "\n".join([format_test(t) for t in tests])

    expansion_name = "Holon Phantoms" if instance.get("expansion") == "holon_phantoms" else "Dragon Frontiers"
    return f'''You are implementing a Pokemon TCG card for the {expansion_name} expansion in Rust.

## Task
EDIT the file `{card_file}` to implement the card below. You MUST:
1. Actually WRITE code to the file - replace the TODO stubs with working implementations
2. Make sure it compiles without errors
3. Make sure all tests pass

## Card to Implement
{card_specs}

## File to Edit
`{card_file}` - This file contains stub functions with TODO comments. You must REPLACE the TODO implementations with actual working code.

## Tests to Pass
{test_descriptions}

## Reference
Look at existing card implementations in `tcg_expansions/src/cg/` (Crystal Guardians) for patterns on how to implement Poke-Powers and attack modifiers.

## CRITICAL Instructions
1. READ the stub file at `{card_file}`
2. READ similar card implementations in tcg_expansions/src/cg/cards/ for patterns
3. EDIT `{card_file}` to replace the TODO stubs with working implementations
4. Run `cargo check --package tcg_expansions` to verify it compiles
5. Run `cargo test --package tcg_expansions -- {instance.get("id", "").replace("-", "_")}` to verify tests pass

You MUST edit the file and write actual code. Do not just describe what to do - actually do it!
'''


def setup_sandbox(instance_id: str, work_dir: Path, quiet: bool = False) -> Path:
    """Set up a sandbox for the coding agent."""
    sandbox_dir = work_dir / "overzealous"

    if not quiet:
        print(f"  Copying repo to {sandbox_dir}...")
    shutil.copytree(
        OVERZEALOUS_REPO,
        sandbox_dir,
        symlinks=True,
        ignore=shutil.ignore_patterns(".git", "target", "*.pyc", "__pycache__"),
    )

    # Use canonical stub from gold/stubs/ if available
    instance = load_instance(instance_id)
    card_file = instance.get("card_file", "")
    if card_file:
        stub_file = BASE_DIR / "gold" / "stubs" / f"{instance_id.replace('-', '_')}.rs"
        stub_path = sandbox_dir / card_file

        if stub_file.exists():
            if not quiet:
                print(f"  Using canonical stub from {stub_file.name}...")
            stub_path.parent.mkdir(parents=True, exist_ok=True)
            stub_path.write_text(stub_file.read_text())

            # Setup expansion module structure if needed (e.g., for HP cards)
            expansion = instance_id.split("-")[0]
            expansion_dir = sandbox_dir / "tcg_expansions" / "src" / expansion
            if expansion != "df":  # DF already has full structure
                card_module = instance_id.replace("-", "_")

                # Create/update cards/mod.rs
                cards_mod_path = expansion_dir / "cards" / "mod.rs"
                if cards_mod_path.exists():
                    content = cards_mod_path.read_text()
                    if f"pub mod {card_module};" not in content:
                        cards_mod_path.write_text(content + f"\npub mod {card_module};")
                else:
                    cards_mod_path.write_text(f"pub mod {card_module};\n")

                # Create hp/mod.rs if needed
                mod_path = expansion_dir / "mod.rs"
                if not mod_path.exists():
                    mod_path.write_text("pub mod cards;\n")
                elif "pub mod cards;" not in mod_path.read_text():
                    mod_path.write_text(mod_path.read_text() + "\npub mod cards;")

                # Update lib.rs if needed
                lib_path = sandbox_dir / "tcg_expansions" / "src" / "lib.rs"
                if lib_path.exists():
                    lib_content = lib_path.read_text()
                    if f"pub mod {expansion};" not in lib_content:
                        lib_path.write_text(lib_content + f"\npub mod {expansion};\n")

    return sandbox_dir


def run_opencode(prompt: str, sandbox_dir: Path, model: str, timeout: int = 300, quiet: bool = False) -> dict:
    """Run OpenCode on the sandbox."""
    if not quiet:
        print(f"  Running OpenCode with {model}...")

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
    )
    return result.returncode == 0, result.stderr


def inject_eval_tests(sandbox_dir: Path, instance_id: str) -> bool:
    """Inject evaluation tests into the sandbox after agent completes."""
    eval_test_file = BASE_DIR / "gold" / "tests" / f"{instance_id.replace('-', '_')}_eval.rs"
    if not eval_test_file.exists():
        return False

    eval_tests = eval_test_file.read_text()
    # Detect expansion from instance_id (hp-*, df-*, etc.)
    expansion = instance_id.split("-")[0]
    card_file = sandbox_dir / "tcg_expansions" / "src" / expansion / "cards" / f"{instance_id.replace('-', '_')}.rs"
    if not card_file.exists():
        return False

    current_content = card_file.read_text()
    eval_module = f'''

// ============================================================================
// EVALUATION TESTS (injected after agent completion)
// ============================================================================

{eval_tests}
'''

    card_file.write_text(current_content + eval_module)
    return True


def run_cargo_test(sandbox_dir: Path, instance_id: str) -> tuple[int, int, str]:
    """Run cargo test and return (passed, total, output)."""
    import re

    test_filter = instance_id.replace("-", "_")
    result = subprocess.run(
        ["cargo", "test", "--package", "tcg_expansions", "--", "--test-threads=1", test_filter],
        cwd=str(sandbox_dir),
        capture_output=True,
        text=True,
        timeout=180,
    )

    output = result.stdout + result.stderr
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
    return passed, total, output


def parse_agent_json_log(raw_output: str) -> str:
    """Parse JSON events from OpenCode into a readable log."""
    lines = []
    for line in raw_output.strip().split("\n"):
        if not line.strip():
            continue
        try:
            event = json.loads(line)
            event_type = event.get("type", "unknown")
            part = event.get("part", {})

            if event_type == "text":
                text = part.get("text", "")
                if text.strip():
                    lines.append(f"[THINKING] {text[:500]}")
            elif event_type == "tool_use":
                tool_name = part.get("tool", "unknown")
                state = part.get("state", {})
                tool_input = state.get("input", {})
                tool_output = state.get("output", "")
                input_str = json.dumps(tool_input, indent=2) if isinstance(tool_input, dict) else str(tool_input)
                if len(input_str) > 500:
                    input_str = input_str[:500] + "..."
                output_str = str(tool_output)
                if len(output_str) > 1000:
                    output_str = output_str[:1000] + "..."
                lines.append(f"\n[TOOL] {tool_name}")
                lines.append(f"  Input: {input_str}")
                lines.append(f"  Output: {output_str}")
            elif event_type == "step_finish":
                cost = part.get("cost", 0)
                tokens = part.get("tokens", {})
                lines.append(f"\n[STEP FINISH] Cost: ${cost:.4f}, Tokens: {tokens}")
        except json.JSONDecodeError:
            if line.strip():
                lines.append(line[:200])

    return "\n".join(lines)


def save_eval_result(instance_id: str, model: str, sandbox_dir: Path, compile_pass: bool,
                     tests_passed: int, tests_total: int, score: float, agent_output: str = "",
                     duration_seconds: float = 0.0) -> Path:
    """Save evaluation result with diff to results directory."""
    results_dir = BASE_DIR / "results"
    results_dir.mkdir(exist_ok=True)

    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    model_safe = model.replace("/", "_").replace(".", "-")
    result_file = results_dir / f"{instance_id}_{model_safe}_{timestamp}.txt"

    stub_file = BASE_DIR / "gold" / "stubs" / f"{instance_id.replace('-', '_')}.rs"
    card_file = sandbox_dir / "tcg_expansions" / "src" / "df" / "cards" / f"{instance_id.replace('-', '_')}.rs"

    stub_content = stub_file.read_text() if stub_file.exists() else ""

    model_content = ""
    if card_file.exists():
        lines = card_file.read_text().split("\n")
        for i, line in enumerate(lines):
            if "EVALUATION TESTS (injected" in line:
                model_content = "\n".join(lines[:i-2])
                break
        else:
            model_content = "\n".join(lines)

    import difflib
    diff = difflib.unified_diff(
        stub_content.splitlines(keepends=True),
        model_content.splitlines(keepends=True),
        fromfile="stub",
        tofile="model_output",
    )
    diff_text = "".join(diff)

    if duration_seconds >= 60:
        duration_str = f"{duration_seconds / 60:.1f} minutes ({duration_seconds:.1f}s)"
    else:
        duration_str = f"{duration_seconds:.1f} seconds"

    result = f"""EngineBench Evaluation Result
=============================
Instance: {instance_id}
Model: {model}
Timestamp: {timestamp}
Duration: {duration_str}

SCORE: {score:.2f}
  - Compile: {'PASS' if compile_pass else 'FAIL'}
  - Tests: {tests_passed}/{tests_total}

ANTI-CHEAT VERIFICATION:
  - Sandbox isolated from engine-bench/gold/
  - Eval tests injected AFTER agent completion
  - Agent worked in: {sandbox_dir}

DIFF (stub -> model output):
{'='*60}
{diff_text if diff_text else '(no changes made)'}
{'='*60}

MODEL OUTPUT (final code):
{'='*60}
{model_content}
{'='*60}

AGENT ACTIVITY LOG (parsed):
{'='*60}
{parse_agent_json_log(agent_output) if agent_output else '(no output captured)'}
{'='*60}
"""

    result_file.write_text(result)
    return result_file


def evaluate_instance(instance_id: str, model: str, timeout: int, quiet: bool = False) -> dict:
    """Evaluate a single instance. Returns result dict."""
    result = {
        "instance_id": instance_id,
        "model": model,
        "compile_pass": False,
        "tests_passed": 0,
        "tests_total": 0,
        "score": 0.0,
        "duration": 0.0,
        "error": None,
    }

    tmpdir = tempfile.mkdtemp()
    try:
        work_dir = Path(tmpdir)
        instance = load_instance(instance_id)

        # Setup sandbox
        sandbox_dir = setup_sandbox(instance_id, work_dir, quiet=quiet)

        # Build prompt
        prompt = build_prompt(instance)

        # Run agent
        start_time = time.time()
        agent_result = run_opencode(prompt, sandbox_dir, model, timeout, quiet=quiet)
        end_time = time.time()
        duration_seconds = end_time - start_time
        result["duration"] = duration_seconds

        # Evaluate
        compile_pass, compile_error = run_cargo_check(sandbox_dir)
        result["compile_pass"] = compile_pass

        tests_passed = 0
        tests_total = 0
        if compile_pass:
            inject_eval_tests(sandbox_dir, instance_id)
            tests_passed, tests_total, test_output = run_cargo_test(sandbox_dir, instance_id)
            result["tests_passed"] = tests_passed
            result["tests_total"] = tests_total

        # Calculate score
        score = 0.0
        if compile_pass:
            score += 0.3
            if tests_total > 0:
                score += 0.7 * (tests_passed / tests_total)
        result["score"] = score

        # Save result
        agent_log = agent_result.get("stdout", "") + "\n" + agent_result.get("stderr", "")
        result_file = save_eval_result(instance_id, model, sandbox_dir, compile_pass,
                                       tests_passed, tests_total, score, agent_log, duration_seconds)
        result["result_file"] = str(result_file)

    except Exception as e:
        result["error"] = str(e)
    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)

    return result


def run_single(args):
    """Run evaluation for a single instance."""
    print(f"\n{'='*60}")
    print(f"EngineBench Local Evaluation")
    print(f"{'='*60}")
    print(f"Model: {args.model}")
    print(f"Instance: {args.instance}")
    print(f"Timeout: {args.timeout}s")
    print(f"{'='*60}\n")

    print("1. Loading instance...")
    instance = load_instance(args.instance)
    print(f"   Card: {instance['cards'][0]['name']}")

    print("\n2. Setting up sandbox...")
    tmpdir = tempfile.mkdtemp()
    try:
        work_dir = Path(tmpdir)
        sandbox_dir = setup_sandbox(args.instance, work_dir)

        print("\n3. Building prompt...")
        prompt = build_prompt(instance)

        print("\n4. Running agent...")
        start_time = time.time()
        agent_result = run_opencode(prompt, sandbox_dir, args.model, args.timeout)
        end_time = time.time()
        duration_seconds = end_time - start_time

        if not agent_result["success"]:
            print(f"   Agent failed: {agent_result['stderr'][:200]}")
        else:
            print(f"   Agent completed in {duration_seconds:.1f}s")

        print("\n5. Evaluating...")
        print("   Running cargo check...")
        compile_pass, compile_error = run_cargo_check(sandbox_dir)
        if compile_pass:
            print("   Compilation passed")
        else:
            print(f"   Compilation failed: {compile_error[:500]}")

        tests_passed = 0
        tests_total = 0
        if compile_pass:
            print("   Injecting eval tests...")
            inject_eval_tests(sandbox_dir, args.instance)

            print("   Running tests...")
            tests_passed, tests_total, test_output = run_cargo_test(sandbox_dir, args.instance)
            print(f"   Tests: {tests_passed}/{tests_total} passed")

        print(f"\n{'='*60}")
        print("RESULTS")
        print(f"{'='*60}")
        if duration_seconds >= 60:
            print(f"  Duration: {duration_seconds / 60:.1f} minutes ({duration_seconds:.1f}s)")
        else:
            print(f"  Duration: {duration_seconds:.1f} seconds")
        print(f"  Compile: {'PASS' if compile_pass else 'FAIL'}")
        print(f"  Tests: {tests_passed}/{tests_total}")

        score = 0.0
        if compile_pass:
            score += 0.3
            if tests_total > 0:
                score += 0.7 * (tests_passed / tests_total)

        print(f"  Score: {score:.2f}")
        print(f"{'='*60}")

        agent_log = agent_result.get("stdout", "") + "\n" + agent_result.get("stderr", "")
        save_eval_result(args.instance, args.model, sandbox_dir, compile_pass,
                        tests_passed, tests_total, score, agent_log, duration_seconds)

        return score
    finally:
        if not args.keep_sandbox:
            shutil.rmtree(tmpdir, ignore_errors=True)


def run_all(args):
    """Run evaluation for all instances in parallel."""
    instances = get_all_instances()

    print(f"\n{'='*60}")
    print(f"EngineBench Parallel Evaluation")
    print(f"{'='*60}")
    print(f"Model: {args.model}")
    print(f"Instances: {len(instances)}")
    print(f"Workers: {args.workers}")
    print(f"Timeout: {args.timeout}s per instance")
    print(f"{'='*60}")
    print(f"\nInstances to evaluate:")
    for inst in instances:
        print(f"  - {inst}")
    print()

    results = []
    start_time = time.time()

    with ThreadPoolExecutor(max_workers=args.workers) as executor:
        futures = {
            executor.submit(evaluate_instance, inst, args.model, args.timeout, quiet=True): inst
            for inst in instances
        }

        for future in as_completed(futures):
            instance_id = futures[future]
            try:
                result = future.result()
                results.append(result)

                # Print progress
                status = "PASS" if result["score"] == 1.0 else f"{result['score']:.2f}"
                duration = result["duration"]
                if duration >= 60:
                    duration_str = f"{duration/60:.1f}m"
                else:
                    duration_str = f"{duration:.0f}s"
                print(f"  [{len(results):2d}/{len(instances)}] {instance_id}: {status} ({duration_str})")

            except Exception as e:
                print(f"  [{len(results):2d}/{len(instances)}] {instance_id}: ERROR - {e}")
                results.append({
                    "instance_id": instance_id,
                    "score": 0.0,
                    "error": str(e),
                    "duration": 0.0,
                })

    total_time = time.time() - start_time

    # Summary
    print(f"\n{'='*60}")
    print("SUMMARY")
    print(f"{'='*60}")
    print(f"Total time: {total_time/60:.1f} minutes")
    print(f"\nResults by instance:")
    print(f"{'Instance':<25} {'Score':>8} {'Tests':>10} {'Duration':>10}")
    print("-" * 55)

    total_score = 0
    for r in sorted(results, key=lambda x: x["instance_id"]):
        inst = r["instance_id"]
        score = r.get("score", 0)
        total_score += score
        tests = f"{r.get('tests_passed', 0)}/{r.get('tests_total', 0)}"
        duration = r.get("duration", 0)
        if duration >= 60:
            duration_str = f"{duration/60:.1f}m"
        else:
            duration_str = f"{duration:.0f}s"
        print(f"{inst:<25} {score:>8.2f} {tests:>10} {duration_str:>10}")

    print("-" * 55)
    avg_score = total_score / len(results) if results else 0
    print(f"{'Average':<25} {avg_score:>8.2f}")
    print(f"{'='*60}")

    return results


def main():
    parser = argparse.ArgumentParser(description="Run local EngineBench evaluation")
    parser.add_argument("--model", type=str, default="openai/gpt-5-nano", help="Model to use")
    parser.add_argument("--instance", type=str, default="df-007-nidoqueen", help="Instance ID (ignored if --all)")
    parser.add_argument("--timeout", type=int, default=300, help="Timeout in seconds per instance")
    parser.add_argument("--keep-sandbox", action="store_true", help="Keep sandbox after eval")
    parser.add_argument("--all", action="store_true", help="Run all instances")
    parser.add_argument("--workers", type=int, default=4, help="Number of parallel workers (with --all)")
    args = parser.parse_args()

    if args.all:
        return run_all(args)
    else:
        return run_single(args)


if __name__ == "__main__":
    main()
