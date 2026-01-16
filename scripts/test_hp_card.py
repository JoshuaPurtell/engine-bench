#!/usr/bin/env python3
"""Test GPT-5-nano on an HP card implementation task.

Usage:
    python scripts/test_hp_card.py --instance hp-017-vileplume
"""

import argparse
import json
import os
from pathlib import Path
from openai import OpenAI

BASE_DIR = Path(__file__).parent.parent


def load_instance(instance_id: str) -> dict:
    """Load instance specification."""
    instance_path = BASE_DIR / "data" / "instances" / "single" / f"{instance_id}.json"
    if not instance_path.exists():
        raise ValueError(f"Instance not found: {instance_id}")
    return json.loads(instance_path.read_text())


def load_stub(instance_id: str) -> str:
    """Load the stub file."""
    stub_path = BASE_DIR / "gold" / "stubs" / f"{instance_id.replace('-', '_')}.rs"
    if not stub_path.exists():
        raise ValueError(f"Stub not found: {stub_path}")
    return stub_path.read_text()


def load_eval_tests(instance_id: str) -> str:
    """Load eval tests."""
    test_path = BASE_DIR / "gold" / "tests" / f"{instance_id.replace('-', '_')}_eval.rs"
    if not test_path.exists():
        return ""
    return test_path.read_text()


def build_prompt(instance: dict, stub: str) -> str:
    """Build the prompt for the model."""
    cards = instance.get("cards", [])
    card_spec = json.dumps(cards[0], indent=2) if cards else "{}"
    
    tests = instance.get("tests", [])
    test_descriptions = "\n".join([
        f"- {t['name']}: {t['description']}"
        for t in tests
    ])

    return f'''You are implementing a Pokemon TCG card for the EX Holon Phantoms expansion in Rust.

## Card Specification
```json
{card_spec}
```

## Current Stub (with TODOs to implement)
```rust
{stub}
```

## Tests Your Implementation Must Pass
{test_descriptions}

## Task
Replace the TODO implementations in the stub with working Rust code. The code must:
1. Implement all the card's abilities and attacks correctly
2. Follow the patterns shown in the stub (function signatures, constants)
3. Compile without errors
4. Pass all the tests

Output ONLY the complete Rust file with your implementations. No explanations, just the code.
'''


def run_model(prompt: str, model: str = "gpt-4o-mini") -> str:
    """Run the model and get the implementation."""
    client = OpenAI()
    
    response = client.chat.completions.create(
        model=model,
        messages=[
            {"role": "system", "content": "You are an expert Rust programmer implementing Pokemon TCG card logic."},
            {"role": "user", "content": prompt}
        ],
        temperature=0.2,
        max_tokens=4000,
    )
    
    return response.choices[0].message.content


def extract_rust_code(response: str) -> str:
    """Extract Rust code from response."""
    # Try to find code block
    if "```rust" in response:
        start = response.find("```rust") + 7
        end = response.find("```", start)
        if end > start:
            return response[start:end].strip()
    elif "```" in response:
        start = response.find("```") + 3
        end = response.find("```", start)
        if end > start:
            return response[start:end].strip()
    return response.strip()


def evaluate_output(output: str, stub: str, eval_tests: str) -> dict:
    """Simple evaluation of the output."""
    result = {
        "has_changes": output != stub,
        "removed_todos": "TODO" not in output or output.count("TODO") < stub.count("TODO"),
        "has_implementations": False,
        "syntax_ok": True,
    }
    
    # Check if implementations were added
    if "fn " in output and ("game" in output or "return" in output):
        result["has_implementations"] = True
    
    # Basic syntax check
    open_braces = output.count("{")
    close_braces = output.count("}")
    if open_braces != close_braces:
        result["syntax_ok"] = False
    
    return result


def main():
    parser = argparse.ArgumentParser(description="Test model on HP card implementation")
    parser.add_argument("--instance", type=str, default="hp-017-vileplume", help="Instance ID")
    parser.add_argument("--model", type=str, default="gpt-4o-mini", help="Model to use")
    args = parser.parse_args()
    
    print(f"\n{'='*60}")
    print(f"Testing {args.model} on {args.instance}")
    print(f"{'='*60}\n")
    
    # Load data
    print("Loading instance data...")
    instance = load_instance(args.instance)
    print(f"  Card: {instance['cards'][0]['name']}")
    
    print("Loading stub...")
    stub = load_stub(args.instance)
    print(f"  Stub size: {len(stub)} chars")
    
    print("Loading eval tests...")
    eval_tests = load_eval_tests(args.instance)
    
    # Build prompt
    print("\nBuilding prompt...")
    prompt = build_prompt(instance, stub)
    print(f"  Prompt size: {len(prompt)} chars")
    
    # Run model
    print(f"\nRunning {args.model}...")
    try:
        response = run_model(prompt, args.model)
        print(f"  Response size: {len(response)} chars")
    except Exception as e:
        print(f"  ERROR: {e}")
        return
    
    # Extract code
    code = extract_rust_code(response)
    
    # Evaluate
    print("\nEvaluating output...")
    result = evaluate_output(code, stub, eval_tests)
    
    print(f"\n{'='*60}")
    print("RESULTS")
    print(f"{'='*60}")
    print(f"  Made changes: {result['has_changes']}")
    print(f"  Removed TODOs: {result['removed_todos']}")
    print(f"  Has implementations: {result['has_implementations']}")
    print(f"  Syntax OK: {result['syntax_ok']}")
    
    # Save output
    output_dir = BASE_DIR / "results"
    output_dir.mkdir(exist_ok=True)
    output_file = output_dir / f"{args.instance}_{args.model.replace('/', '_')}_output.rs"
    output_file.write_text(code)
    print(f"\n  Output saved to: {output_file}")
    
    # Show snippet
    print(f"\n{'='*60}")
    print("OUTPUT SNIPPET (first 80 lines):")
    print(f"{'='*60}")
    lines = code.split('\n')[:80]
    print('\n'.join(lines))
    if len(code.split('\n')) > 80:
        print(f"\n... ({len(code.split(chr(10))) - 80} more lines)")


if __name__ == "__main__":
    main()
