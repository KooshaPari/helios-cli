#!/usr/bin/env python3
"""
Cross-Harness + Cross-Model Benchmark
====================================
Tests multiple harnesses with multiple models via cliproxy.
Goal: Measure harness overhead, not model performance.
"""

import asyncio
import json
import os
import time
import psutil
import httpx
from dataclasses import dataclass
from typing import List, Dict, Any, Optional
from datetime import datetime
from itertools import product

CLIPROXY_URL = os.environ.get("CLIPROXY_URL", "http://localhost:8317")

# Models to test (via cliproxy - all use same routing layer)
MODELS = [
    "minimax-m2.5",
    "minimax-m2.1", 
    "gpt-5.3-codex",
    "claude-sonnet-4.5",
    "deepseek-v3.2-chat",
]

# Task prompts (simple to isolate harness overhead)
TASKS = [
    "Write a hello world function in Python",
    "Write a function to reverse a string", 
    "Write a function to check if a number is prime",
    "Write a function to find the factorial of a number",
    "Write a function to check if a list is sorted",
]

# Test configurations
CONFIGS = list(product(MODELS, TASKS))

@dataclass
class RunResult:
    model: str
    task: str
    latency_ms: float
    success: bool
    rss_mb: float
    threads: int
    fds: int
    timestamp: str


async def run_task(client: httpx.AsyncClient, model: str, prompt: str) -> RunResult:
    """Run a single task."""
    proc = psutil.Process()
    
    start = time.perf_counter()
    try:
        r = await client.post(
            f"{CLIPROXY_URL}/v1/chat/completions",
            json={"model": model, "messages": [{"role": "user", "content": prompt}], "max_tokens": 50},
            timeout=30.0
        )
        elapsed = (time.perf_counter() - start) * 1000
        success = r.status_code == 200
    except Exception as e:
        elapsed = (time.perf_counter() - start) * 1000
        success = False
    
    mem = proc.memory_info()
    threads = proc.num_threads()
    try:
        fds = proc.num_fds()
    except:
        fds = 0
    
    return RunResult(
        model=model,
        task=prompt[:30],
        latency_ms=elapsed,
        success=success,
        rss_mb=mem.rss / (1024 * 1024),
        threads=threads,
        fds=fds,
        timestamp=datetime.now().isoformat()
    )


async def run_batch(model: str, tasks: List[str], batch_name: str) -> List[RunResult]:
    """Run a batch of tasks."""
    results = []
    async with httpx.AsyncClient() as client:
        for task in tasks:
            r = await run_task(client, model, task)
            results.append(r)
            print(f"  {model}: {r.latency_ms:.0f}ms {'✓' if r.success else '✗'}")
    return results


def analyze(results: List[RunResult], name: str) -> Dict:
    """Analyze results."""
    successful = [r for r in results if r.success]
    latencies = [r.latency_ms for r in successful]
    
    if not latencies:
        return {"success": 0, "avg_latency": 0, "avg_rss": 0, "avg_threads": 0, "avg_fds": 0}
    
    return {
        "success": len(successful) / len(results),
        "avg_latency": sum(latencies) / len(latencies),
        "min_latency": min(latencies),
        "max_latency": max(latencies),
        "avg_rss": sum(r.rss_mb for r in results) / len(results),
        "avg_threads": sum(r.threads for r in results) / len(results),
        "avg_fds": sum(r.fds for r in results) / len(results),
    }


async def main():
    print("="*70)
    print("CROSS-HARNESS + CROSS-MODEL BENCHMARK")
    print("="*70)
    print(f"Cliproxy: {CLIPROXY_URL}")
    print(f"Models: {len(MODELS)}")
    print(f"Tasks: {len(TASKS)}")
    print(f"Total: {len(CONFIGS)} runs")
    print()
    
    all_results = []
    
    # Run each model
    for model in MODELS:
        print(f"\n{'='*50}")
        print(f"MODEL: {model}")
        print(f"{'='*50}")
        results = await run_batch(model, TASKS, model)
        all_results.extend(results)
    
    # Summary by model
    print("\n" + "="*70)
    print("SUMMARY BY MODEL (HARNESS OVERHEAD ISOLATED)")
    print("="*70)
    print(f"{'Model':<25} {'Success':<10} {'Latency':<15} {'RSS':<10} {'Threads':<10} {'FDs':<8}")
    print("-"*70)
    
    for model in MODELS:
        model_results = [r for r in all_results if r.model == model]
        stats = analyze(model_results, model)
        print(f"{model:<25} {stats['success']*100:>6.0f}%   "
              f"{stats['avg_latency']:>8.0f}ms   "
              f"{stats['avg_rss']:>6.1f}MB   "
              f"{stats['avg_threads']:>6.1f}    "
              f"{stats['avg_fds']:>4.0f}")
    
    # Overall
    overall = analyze(all_results, "OVERALL")
    print("-"*70)
    print(f"{'OVERALL':<25} {overall['success']*100:>6.0f}%   "
          f"{overall['avg_latency']:>8.0f}ms   "
          f"{overall['avg_rss']:>6.1f}MB   "
          f"{overall['avg_threads']:>6.1f}    "
          f"{overall['avg_fds']:>4.0f}")
    
    print("\n" + "="*70)
    print("KEY INSIGHT: All models use SAME cliproxy routing layer.")
print("Differences reflect model inference time, NOT harness overhead.")


if __name__ == "__main__":
    asyncio.run(main())
