#!/usr/bin/env python3
"""EngineBench evaluation runner.

Runs the benchmark against a model and collects results.

Usage:
    python -m src.run_eval --model gpt-4.1-mini --instances all
    python -m src.run_eval --model deepseek-chat --instances df-007-flygon
"""

from __future__ import annotations

import argparse
import asyncio
import json
import sys
from dataclasses import asdict, dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Any

import httpx

# Default task app URL
DEFAULT_TASK_URL = "http://localhost:8017"


@dataclass
class EvalConfig:
    """Configuration for an evaluation run."""

    model: str
    temperature: float = 0.2
    max_tokens: int = 4096
    provider: str = "openai"
    mode: str = "single_card"
    instances: list[str] = field(default_factory=list)
    parallelism: int = 1
    sandbox_path: str | None = None


@dataclass
class InstanceResult:
    """Result for a single instance."""

    instance_id: str
    score: float
    compile_pass: bool
    tests_passed: int
    tests_total: int
    gold_similarity: float
    error: str | None = None
    latency_ms: float = 0.0


@dataclass
class EvalResults:
    """Full evaluation results."""

    config: EvalConfig
    started_at: str
    completed_at: str | None = None
    results: list[InstanceResult] = field(default_factory=list)

    @property
    def total_instances(self) -> int:
        return len(self.results)

    @property
    def passed_instances(self) -> int:
        return sum(1 for r in self.results if r.compile_pass and r.tests_passed > 0)

    @property
    def average_score(self) -> float:
        if not self.results:
            return 0.0
        return sum(r.score for r in self.results) / len(self.results)

    @property
    def compile_rate(self) -> float:
        if not self.results:
            return 0.0
        return sum(1 for r in self.results if r.compile_pass) / len(self.results)

    def to_dict(self) -> dict[str, Any]:
        return {
            "config": asdict(self.config),
            "started_at": self.started_at,
            "completed_at": self.completed_at,
            "results": [asdict(r) for r in self.results],
            "summary": {
                "total_instances": self.total_instances,
                "passed_instances": self.passed_instances,
                "average_score": self.average_score,
                "compile_rate": self.compile_rate,
            },
        }


class EvalRunner:
    """Runs EngineBench evaluations."""

    def __init__(self, task_url: str = DEFAULT_TASK_URL):
        self.task_url = task_url
        self.client = httpx.AsyncClient(timeout=300.0)

    async def close(self):
        await self.client.aclose()

    async def get_instances(self, mode: str) -> list[str]:
        """Get available instances from the task app."""
        response = await self.client.get(f"{self.task_url}/info")
        response.raise_for_status()
        info = response.json()

        if mode == "single_card":
            return info.get("instances", {}).get("single", [])
        elif mode == "full_deck":
            return info.get("instances", {}).get("deck", [])
        return []

    async def run_instance(
        self,
        instance_id: str,
        config: EvalConfig,
    ) -> InstanceResult:
        """Run a single instance."""
        import time

        start_time = time.time()

        try:
            response = await self.client.post(
                f"{self.task_url}/rollout",
                json={
                    "seed": 0,
                    "env_name": "engine_bench",
                    "env_config": {
                        "instance_id": instance_id,
                        "mode": config.mode,
                        "sandbox_path": config.sandbox_path,
                    },
                    "policy_config": {
                        "model": config.model,
                        "temperature": config.temperature,
                        "max_tokens": config.max_tokens,
                        "provider": config.provider,
                    },
                },
            )
            response.raise_for_status()
            result = response.json()

            latency_ms = (time.time() - start_time) * 1000

            return InstanceResult(
                instance_id=instance_id,
                score=result.get("score", 0.0),
                compile_pass=result.get("metrics", {}).get("compile_pass", False),
                tests_passed=result.get("metrics", {}).get("tests_passed", 0),
                tests_total=result.get("metrics", {}).get("tests_total", 0),
                gold_similarity=result.get("metrics", {}).get("gold_similarity", 0.0),
                error=result.get("error"),
                latency_ms=latency_ms,
            )

        except Exception as e:
            latency_ms = (time.time() - start_time) * 1000
            return InstanceResult(
                instance_id=instance_id,
                score=0.0,
                compile_pass=False,
                tests_passed=0,
                tests_total=0,
                gold_similarity=0.0,
                error=str(e),
                latency_ms=latency_ms,
            )

    async def run_eval(self, config: EvalConfig) -> EvalResults:
        """Run a full evaluation."""
        results = EvalResults(
            config=config,
            started_at=datetime.now().isoformat(),
        )

        # Get instances to run
        if not config.instances or config.instances == ["all"]:
            instances = await self.get_instances(config.mode)
        else:
            instances = config.instances

        print(f"Running evaluation on {len(instances)} instances...")
        print(f"Model: {config.model}")
        print(f"Mode: {config.mode}")
        print("-" * 50)

        # Run instances with parallelism
        if config.parallelism > 1:
            # Run in parallel batches
            for i in range(0, len(instances), config.parallelism):
                batch = instances[i : i + config.parallelism]
                tasks = [self.run_instance(inst, config) for inst in batch]
                batch_results = await asyncio.gather(*tasks)

                for result in batch_results:
                    results.results.append(result)
                    self._print_result(result)
        else:
            # Run sequentially
            for instance_id in instances:
                result = await self.run_instance(instance_id, config)
                results.results.append(result)
                self._print_result(result)

        results.completed_at = datetime.now().isoformat()

        # Print summary
        print("-" * 50)
        print("SUMMARY")
        print(f"  Total instances: {results.total_instances}")
        print(f"  Passed instances: {results.passed_instances}")
        print(f"  Average score: {results.average_score:.3f}")
        print(f"  Compile rate: {results.compile_rate:.1%}")

        return results

    def _print_result(self, result: InstanceResult):
        """Print a single result."""
        status = "+" if result.compile_pass else "x"
        tests = f"{result.tests_passed}/{result.tests_total}"
        print(
            f"  {status} {result.instance_id}: "
            f"score={result.score:.3f}, tests={tests}, "
            f"gold={result.gold_similarity:.2f}, "
            f"latency={result.latency_ms:.0f}ms"
        )
        if result.error:
            print(f"    Error: {result.error[:100]}")


async def main():
    parser = argparse.ArgumentParser(description="EngineBench Evaluation Runner")
    parser.add_argument("--model", type=str, default="gpt-4.1-mini", help="Model to use")
    parser.add_argument("--temperature", type=float, default=0.2, help="Temperature")
    parser.add_argument("--max-tokens", type=int, default=4096, help="Max tokens")
    parser.add_argument("--provider", type=str, default="openai", help="Provider")
    parser.add_argument("--mode", type=str, default="single_card", help="Benchmark mode")
    parser.add_argument(
        "--instances",
        type=str,
        nargs="+",
        default=["all"],
        help="Instance IDs to run (or 'all')",
    )
    parser.add_argument("--parallelism", type=int, default=1, help="Parallelism")
    parser.add_argument("--task-url", type=str, default=DEFAULT_TASK_URL, help="Task app URL")
    parser.add_argument("--sandbox-path", type=str, help="Path to sandbox repo")
    parser.add_argument("--output", type=str, help="Output JSON file")

    args = parser.parse_args()

    config = EvalConfig(
        model=args.model,
        temperature=args.temperature,
        max_tokens=args.max_tokens,
        provider=args.provider,
        mode=args.mode,
        instances=args.instances,
        parallelism=args.parallelism,
        sandbox_path=args.sandbox_path,
    )

    runner = EvalRunner(task_url=args.task_url)

    try:
        results = await runner.run_eval(config)

        # Save results
        if args.output:
            output_path = Path(args.output)
            output_path.write_text(json.dumps(results.to_dict(), indent=2))
            print(f"\nResults saved to {output_path}")

    finally:
        await runner.close()


if __name__ == "__main__":
    asyncio.run(main())
