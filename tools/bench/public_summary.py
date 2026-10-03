"""Summary table for tools/bench/public.sh: one row per scenario and tool from OUT/<id>.json (hyperfine exports).

    python3 tools/bench/public_summary.py OUT
"""

import json
import statistics
import sys
from pathlib import Path


def fmt_time(seconds):
    if seconds < 0.01:
        return f"{seconds * 1000:.1f} ms"
    return f"{seconds * 1000:.0f} ms" if seconds < 1 else f"{seconds:.2f} s"


def main(out):
    out = Path(out)
    env = out / "env.md"
    if env.exists():
        print(env.read_text().rstrip() + "\n")
    print("| Scenario | Tool | Median | ± stddev | User+sys (mean) | Max RSS (median) | ktrs speedup |")
    print("|---|---|--:|--:|--:|--:|--:|")
    warnings = []
    for line in (out / "scenarios.tsv").read_text().splitlines():
        sid, label = line.split("\t", 1)
        results = json.loads((out / f"{sid}.json").read_text())["results"]
        ours = results[0]["median"]
        for r in results:
            rss = r.get("memory_usage_byte")
            rss = f"{statistics.median(rss) / 1e6:.0f} MB" if rss else "—"
            speedup = "" if r is results[0] else f"**{r['median'] / ours:.1f}x**"
            stddev = fmt_time(r["stddev"]) if r.get("stddev") is not None else "—"
            print(f"| {label} | {r['command']} | {fmt_time(r['median'])} | {stddev} "
                  f"| {r['user'] + r['system']:.2f} s | {rss} | {speedup} |")
            odd = sorted({c for c in r.get("exit_codes", []) if c not in (0, 1)})
            if odd:
                warnings.append(f"{sid} / {r['command']}: exit codes {odd}")
    for w in warnings:
        print(f"\nWARNING {w}")


if __name__ == "__main__":
    main(sys.argv[1])
