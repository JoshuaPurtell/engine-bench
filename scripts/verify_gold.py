#!/usr/bin/env python3
"""Verify gold patches pass all evaluation tests.

This script:
1. Creates a sandbox from overzealous
2. Applies the stub for a card
3. Applies the gold patch
4. Injects eval tests
5. Runs cargo test
6. Reports results

Usage:
    python scripts/verify_gold.py --instance df-007-nidoqueen
    python scripts/verify_gold.py --all
"""

import argparse
import shutil
import subprocess
import tempfile
import re
from pathlib import Path

BASE_DIR = Path(__file__).parent.parent
GOLD_DIR = BASE_DIR / "gold"
OVERZEALOUS_REPO = Path.home() / "Documents" / "GitHub" / "overzealous"


def get_instances() -> list[str]:
    """Get all available instance IDs."""
    patches_dir = GOLD_DIR / "patches"
    if not patches_dir.exists():
        return []
    return [p.stem for p in patches_dir.glob("*.patch")]


def verify_instance(instance_id: str, verbose: bool = False) -> dict:
    """Verify a gold patch passes all tests."""
    card_file = instance_id.replace("-", "_")
    stub_file = GOLD_DIR / "stubs" / f"{card_file}.rs"
    patch_file = GOLD_DIR / "patches" / f"{card_file}.patch"
    tests_file = GOLD_DIR / "tests" / f"{card_file}_eval.rs"

    result = {
        "instance_id": instance_id,
        "stub_exists": stub_file.exists(),
        "patch_exists": patch_file.exists(),
        "tests_exist": tests_file.exists(),
        "compile_pass": False,
        "tests_passed": 0,
        "tests_total": 0,
        "error": None,
    }

    if not stub_file.exists():
        result["error"] = f"Stub not found: {stub_file}"
        return result
    if not patch_file.exists():
        result["error"] = f"Patch not found: {patch_file}"
        return result
    if not tests_file.exists():
        result["error"] = f"Tests not found: {tests_file}"
        return result

    with tempfile.TemporaryDirectory() as tmpdir:
        sandbox_dir = Path(tmpdir) / "overzealous"

        # 1. Copy repo (excluding .git and target)
        print(f"  Creating sandbox...")
        shutil.copytree(
            OVERZEALOUS_REPO,
            sandbox_dir,
            symlinks=True,
            ignore=shutil.ignore_patterns(".git", "target", "*.pyc", "__pycache__"),
        )

        # 2. Apply stub
        card_path = sandbox_dir / "tcg_expansions" / "src" / "df" / "cards" / f"{card_file}.rs"
        print(f"  Applying stub...")
        shutil.copy(stub_file, card_path)

        # 3. Apply gold patch
        print(f"  Applying gold patch...")
        proc = subprocess.run(
            ["patch", "-p0", "-i", str(patch_file)],
            cwd=str(GOLD_DIR),
            capture_output=True,
            text=True,
        )

        # Copy patched file to sandbox
        patched_file = GOLD_DIR / "stubs" / f"{card_file}.rs"
        if patched_file.exists():
            shutil.copy(patched_file, card_path)
            # Restore original stub
            subprocess.run(
                ["patch", "-R", "-p0", "-i", str(patch_file)],
                cwd=str(GOLD_DIR),
                capture_output=True,
            )
        else:
            # Alternative: copy gold implementation directly
            gold_impl = GOLD_DIR / "implementations" / f"{card_file}.rs"
            if gold_impl.exists():
                shutil.copy(gold_impl, card_path)

        # 4. Inject eval tests
        print(f"  Injecting eval tests...")
        current_content = card_path.read_text()
        eval_tests = tests_file.read_text()
        card_path.write_text(current_content + "\n\n" + eval_tests)

        # 5. Run cargo check
        print(f"  Running cargo check...")
        proc = subprocess.run(
            ["cargo", "check", "--package", "tcg_expansions"],
            cwd=str(sandbox_dir),
            capture_output=True,
            text=True,
        )

        if proc.returncode != 0:
            result["error"] = f"Compilation failed:\n{proc.stderr[:500]}"
            if verbose:
                print(f"  Compile error: {proc.stderr[:500]}")
            return result

        result["compile_pass"] = True

        # 6. Run tests
        print(f"  Running tests...")
        proc = subprocess.run(
            ["cargo", "test", "--package", "tcg_expansions", "--", card_file],
            cwd=str(sandbox_dir),
            capture_output=True,
            text=True,
            timeout=120,
        )

        output = proc.stdout + proc.stderr
        if verbose:
            print(f"  Test output:\n{output}")

        # Parse results
        for line in output.split("\n"):
            if "test result:" in line:
                match_passed = re.search(r"(\d+) passed", line)
                match_failed = re.search(r"(\d+) failed", line)
                if match_passed:
                    result["tests_passed"] = int(match_passed.group(1))
                if match_failed:
                    result["tests_total"] = result["tests_passed"] + int(match_failed.group(1))
                else:
                    result["tests_total"] = result["tests_passed"]

    return result


def main():
    parser = argparse.ArgumentParser(description="Verify gold patches")
    parser.add_argument("--instance", type=str, help="Instance ID to verify")
    parser.add_argument("--all", action="store_true", help="Verify all instances")
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")
    args = parser.parse_args()

    if args.all:
        instances = get_instances()
    elif args.instance:
        instances = [args.instance]
    else:
        instances = get_instances()
        if not instances:
            print("No instances found. Use --instance to specify one.")
            return

    print(f"\n{'='*60}")
    print("Gold Patch Verification")
    print(f"{'='*60}\n")

    all_passed = True
    for instance_id in instances:
        print(f"Verifying {instance_id}...")
        result = verify_instance(instance_id, args.verbose)

        if result["error"]:
            print(f"  ERROR: {result['error']}")
            all_passed = False
        elif result["compile_pass"] and result["tests_passed"] == result["tests_total"]:
            print(f"  PASS: {result['tests_passed']}/{result['tests_total']} tests")
        else:
            print(f"  FAIL: {result['tests_passed']}/{result['tests_total']} tests")
            all_passed = False
        print()

    print(f"{'='*60}")
    if all_passed:
        print("All gold patches verified!")
    else:
        print("Some gold patches failed verification.")
    print(f"{'='*60}\n")


if __name__ == "__main__":
    main()
