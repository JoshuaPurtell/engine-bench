#!/usr/bin/env python3
"""Show seed-by-seed score comparisons for archived/rejected candidates.

This script helps visualize why candidates were rejected by showing
seed-by-seed score comparisons between the rejected candidate and
the dominating candidate.

Usage:
    python scripts/show_seed_comparisons.py --candidate-id <id> --dominating-id <id> --results-dir results/
    python scripts/show_seed_comparisons.py --from-log <log_file>
"""

from __future__ import annotations

import argparse
import json
import re
from collections import defaultdict
from pathlib import Path
from typing import Any

BASE_DIR = Path(__file__).parent.parent


def parse_archive_log(log_file: Path) -> list[dict[str, Any]]:
    """Parse log file for ARCHIVE messages with candidate comparisons."""
    archive_entries = []
    
    if not log_file.exists():
        print(f"Log file not found: {log_file}")
        return archive_entries
    
    content = log_file.read_text()
    
    # Pattern: ARCHIVE: Candidate rejected (dominated by existing): reward=0.200 vs 0.400
    pattern = r'ARCHIVE:\s+Candidate rejected.*?reward=([\d.]+)\s+vs\s+([\d.]+)'
    
    for match in re.finditer(pattern, content):
        archive_entries.append({
            "rejected_reward": float(match.group(1)),
            "dominating_reward": float(match.group(2)),
            "line": match.group(0),
            "context": content[max(0, match.start() - 200):match.end() + 200],
        })
    
    return archive_entries


def load_seed_results(results_dir: Path, candidate_id: str) -> dict[int, float]:
    """Load seed-by-seed results for a candidate from results directory."""
    seed_scores = {}
    
    # Look for JSON files with seed information
    for result_file in results_dir.glob("*.json"):
        try:
            data = json.loads(result_file.read_text())
            
            # Check if this file contains results for the candidate
            if "results" in data:
                for result in data.get("results", []):
                    if result.get("instance_id") == candidate_id:
                        # Try to extract seed from result or filename
                        seed = result.get("seed", 0)
                        score = result.get("score", result.get("outcome_reward", 0.0))
                        seed_scores[seed] = score
            
            # Check trajectories for seed-specific data
            if "trajectories" in data:
                for traj in data.get("trajectories", []):
                    if traj.get("seed") is not None:
                        seed = traj["seed"]
                        score = traj.get("final", {}).get("info", {}).get("score", 0.0)
                        if score > 0:
                            seed_scores[seed] = score
        except Exception:
            continue
    
    # Also check for seed information in rollout responses
    # Look for files that might contain seed-specific data
    for result_file in results_dir.glob(f"*{candidate_id}*"):
        if result_file.suffix == ".json":
            try:
                data = json.loads(result_file.read_text())
                if "seed" in data:
                    seed = data["seed"]
                    score = data.get("metrics", {}).get("outcome_reward", 
                            data.get("metrics", {}).get("reward_mean", 0.0))
                    seed_scores[seed] = score
            except Exception:
                continue
    
    # Check for seed-specific result files (e.g., from GEPA runs)
    # Pattern: candidate_id_seed_*.json or similar
    for result_file in results_dir.rglob(f"*{candidate_id}*seed*.json"):
        try:
            data = json.loads(result_file.read_text())
            seed = data.get("seed", 0)
            score = data.get("metrics", {}).get("outcome_reward",
                    data.get("metrics", {}).get("reward_mean", 0.0))
            seed_scores[seed] = score
        except Exception:
            continue
    
    return seed_scores


def compare_seeds(
    candidate_scores: dict[int, float],
    dominating_scores: dict[int, float],
    candidate_id: str,
    dominating_id: str,
) -> None:
    """Display seed-by-seed comparison between two candidates."""
    print(f"\n{'='*80}")
    print(f"Seed-by-Seed Score Comparison")
    print(f"{'='*80}")
    print(f"Candidate: {candidate_id}")
    print(f"Dominating: {dominating_id}")
    print(f"{'='*80}\n")
    
    # Get all seeds that appear in either candidate
    all_seeds = sorted(set(candidate_scores.keys()) | set(dominating_scores.keys()))
    
    if not all_seeds:
        print("No seed-specific data found. Showing aggregate comparison:")
        if candidate_scores:
            avg_candidate = sum(candidate_scores.values()) / len(candidate_scores)
            print(f"  Candidate average: {avg_candidate:.3f}")
        if dominating_scores:
            avg_dominating = sum(dominating_scores.values()) / len(dominating_scores)
            print(f"  Dominating average: {avg_dominating:.3f}")
        return
    
    # Header
    print(f"{'Seed':<10} {'Candidate':<15} {'Dominating':<15} {'Diff':<15} {'Winner':<10}")
    print("-" * 80)
    
    candidate_wins = 0
    dominating_wins = 0
    ties = 0
    
    for seed in all_seeds:
        candidate_score = candidate_scores.get(seed, None)
        dominating_score = dominating_scores.get(seed, None)
        
        if candidate_score is None:
            candidate_str = "-"
            diff_str = "-"
            winner = "Dominating"
            dominating_wins += 1
        elif dominating_score is None:
            candidate_str = f"{candidate_score:.3f}"
            diff_str = "-"
            winner = "Candidate"
            candidate_wins += 1
        else:
            candidate_str = f"{candidate_score:.3f}"
            diff = candidate_score - dominating_score
            diff_str = f"{diff:+.3f}"
            
            if candidate_score > dominating_score:
                winner = "Candidate"
                candidate_wins += 1
            elif dominating_score > candidate_score:
                winner = "Dominating"
                dominating_wins += 1
            else:
                winner = "Tie"
                ties += 1
        
        dominating_str = f"{dominating_score:.3f}" if dominating_score is not None else "-"
        
        print(f"{seed:<10} {candidate_str:<15} {dominating_str:<15} {diff_str:<15} {winner:<10}")
    
    print("-" * 80)
    print(f"\nSummary:")
    print(f"  Total seeds compared: {len(all_seeds)}")
    print(f"  Candidate wins: {candidate_wins}")
    print(f"  Dominating wins: {dominating_wins}")
    print(f"  Ties: {ties}")
    
    if candidate_scores and dominating_scores:
        avg_candidate = sum(candidate_scores.values()) / len(candidate_scores)
        avg_dominating = sum(dominating_scores.values()) / len(dominating_scores)
        print(f"\n  Average scores:")
        print(f"    Candidate: {avg_candidate:.3f}")
        print(f"    Dominating: {avg_dominating:.3f}")
        print(f"    Difference: {avg_candidate - avg_dominating:+.3f}")


def main():
    parser = argparse.ArgumentParser(
        description="Show seed-by-seed score comparisons for archived candidates"
    )
    parser.add_argument(
        "--candidate-id",
        type=str,
        help="ID of the rejected candidate",
    )
    parser.add_argument(
        "--dominating-id",
        type=str,
        help="ID of the dominating candidate",
    )
    parser.add_argument(
        "--results-dir",
        type=Path,
        default=BASE_DIR / "results",
        help="Directory containing evaluation results",
    )
    parser.add_argument(
        "--from-log",
        type=Path,
        help="Parse ARCHIVE messages from a log file",
    )
    parser.add_argument(
        "--seed-range",
        type=str,
        help="Seed range to analyze (e.g., '0-100')",
    )
    
    args = parser.parse_args()
    
    if args.from_log:
        # Parse log file for archive entries
        entries = parse_archive_log(args.from_log)
        if not entries:
            print(f"No ARCHIVE entries found in {args.from_log}")
            return
        
        print(f"Found {len(entries)} ARCHIVE entries:")
        for i, entry in enumerate(entries, 1):
            print(f"\n{i}. {entry['line']}")
            print(f"   Rejected reward: {entry['rejected_reward']:.3f}")
            print(f"   Dominating reward: {entry['dominating_reward']:.3f}")
        
        if args.candidate_id and args.dominating_id:
            # Load seed-specific results
            candidate_scores = load_seed_results(args.results_dir, args.candidate_id)
            dominating_scores = load_seed_results(args.results_dir, args.dominating_id)
            
            compare_seeds(
                candidate_scores,
                dominating_scores,
                args.candidate_id,
                args.dominating_id,
            )
    
    elif args.candidate_id and args.dominating_id:
        # Direct comparison mode
        candidate_scores = load_seed_results(args.results_dir, args.candidate_id)
        dominating_scores = load_seed_results(args.results_dir, args.dominating_id)
        
        compare_seeds(
            candidate_scores,
            dominating_scores,
            args.candidate_id,
            args.dominating_id,
        )
    else:
        parser.print_help()
        print("\nError: Either provide --from-log or both --candidate-id and --dominating-id")


if __name__ == "__main__":
    main()
