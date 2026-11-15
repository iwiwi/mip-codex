#!/usr/bin/env python3
"""Differential LP tests. HiGHS runs only here as an independent oracle."""
import argparse
import hashlib
import json
from pathlib import Path
import random
import shutil
import subprocess
import tempfile
import highspy


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--cases", type=int, default=1000)
    parser.add_argument("--solver", default="target/debug/mip-codex")
    parser.add_argument("--seed", type=int, default=2026111502)
    parser.add_argument("--max-cols", type=int, default=15)
    parser.add_argument("--max-rows", type=int, default=20)
    parser.add_argument("--bounded", action="store_true", help="give every variable finite bounds")
    parser.add_argument("--allow-unresolved", action="store_true", help="report numerical/resource limits without accepting an incorrect terminal claim")
    args = parser.parse_args()
    rng = random.Random(args.seed)
    failures = []
    statuses = {}
    oracle_fallbacks = 0
    unresolved = 0
    with tempfile.TemporaryDirectory() as directory:
        frozen_solver = Path(directory) / "solver"
        shutil.copy2(Path(args.solver).resolve(), frozen_solver)
        binary_sha256 = hashlib.sha256(frozen_solver.read_bytes()).hexdigest()
        for case in range(args.cases):
            n, m = rng.randrange(1, args.max_cols + 1), rng.randrange(0, args.max_rows + 1)
            costs = [rng.randrange(-9, 10) for _ in range(n)]
            a = [[rng.randrange(-7, 8) if rng.random() < .6 else 0 for _ in range(n)] for _ in range(m)]
            witness = [rng.randrange(-4, 5) for _ in range(n)]
            kinds = [rng.choice("LLGEE") for _ in range(m)]
            bounds = []
            for j in range(n):
                lo, up = witness[j] - rng.randrange(0, 6), witness[j] + rng.randrange(0, 6)
                kind = rng.randrange(4)
                if args.bounded:
                    kind = 0
                if kind == 3:
                    bounds += [f" FR b x{j}"]
                elif kind == 2:
                    bounds += [f" MI b x{j}"]
                if kind in (0, 1):
                    bounds += [f" LO b x{j} {lo}"]
                if kind in (0, 2):
                    bounds += [f" UP b x{j} {up}"]
            rhs = []
            for i in range(m):
                activity = sum(c * x for c, x in zip(a[i], witness))
                # 80% known feasible instances; the remaining cases have arbitrary RHS.
                if case % 5:
                    v = activity + (rng.randrange(0, 6) if kinds[i] == "L" else -rng.randrange(0, 6) if kinds[i] == "G" else 0)
                else:
                    v = rng.randrange(-25, 26)
                rhs += [f" rhs r{i} {v}"]
            lines = ["NAME random", "OBJSENSE", rng.choice([" MIN", " MAX"]), "ROWS", " N obj"]
            lines += [f" {kind} r{i}" for i, kind in enumerate(kinds)] + ["COLUMNS"]
            for j in range(n):
                lines += [f" x{j} obj {costs[j]}"]
                lines += [f" x{j} r{i} {a[i][j]}" for i in range(m) if a[i][j]]
            lines += ["RHS", f" rhs obj {rng.randrange(-8, 9)}"] + rhs + ["BOUNDS"] + bounds + ["ENDATA"]
            path = Path(directory) / "test.mps"
            path.write_text("\n".join(lines) + "\n")
            oracle = highspy.Highs()
            oracle.setOptionValue("output_flag", False)
            oracle.setOptionValue("threads", 1)
            oracle.setOptionValue("allow_unbounded_or_infeasible", False)
            read_status = oracle.readModel(str(path))
            if read_status == highspy.HighsStatus.kError:
                Path("results").mkdir(exist_ok=True)
                Path("results/oracle-read-error.mps").write_text(path.read_text())
                raise RuntimeError(f"HiGHS failed to read case {case}")
            oracle.run()
            known = {highspy.HighsModelStatus.kOptimal: "optimal",
                        highspy.HighsModelStatus.kInfeasible: "infeasible",
                        highspy.HighsModelStatus.kUnbounded: "unbounded_relaxation"}
            if oracle.getModelStatus() not in known:
                # Degenerate infeasible systems can end in kUnknown in the default
                # dual simplex. Keep this event visible and ask the independent IPM.
                oracle_fallbacks += 1
                oracle = highspy.Highs()
                oracle.setOptionValue("output_flag", False)
                oracle.setOptionValue("threads", 1)
                oracle.setOptionValue("solver", "ipm")
                oracle.readModel(str(path))
                oracle.run()
            expected = known.get(oracle.getModelStatus())
            run = subprocess.run([str(frozen_solver), "solve", str(path), "5"], capture_output=True, text=True)
            try:
                got = json.loads(run.stdout)
                valid = run.returncode == 0 and got["status"] == expected
                if got["status"] in ("numerical_failure", "resource_limit", "time_limit"):
                    unresolved += 1
                    valid = bool(args.allow_unresolved and run.returncode == 0)
                elif expected == "optimal":
                    ref = oracle.getObjectiveValue()
                    valid &= got["objective"] is not None and abs(got["objective"] - ref) <= 1e-6 * max(1, abs(ref))
                    lp = oracle.getLp()
                    x = got["values"]
                    valid &= len(x) == lp.num_col_ and got["variable_names"] == lp.col_names_
                    valid &= all(lo - 1e-6 <= v <= up + 1e-6 for lo, v, up in zip(lp.col_lower_, x, lp.col_upper_))
                    activity = [0.] * lp.num_row_
                    matrix = lp.a_matrix_
                    assert matrix.format_ == highspy.MatrixFormat.kColwise
                    for j, v in enumerate(x):
                        for k in range(matrix.start_[j], matrix.start_[j + 1]):
                            activity[matrix.index_[k]] += matrix.value_[k] * v
                    valid &= all(lo - 1e-6 <= v <= up + 1e-6 for lo, v, up in zip(lp.row_lower_, activity, lp.row_upper_))
                statuses[got["status"]] = statuses.get(got["status"], 0) + 1
            except Exception as error:
                got = {"error": str(error), "stderr": run.stderr}
                valid = False
            if not valid:
                failure = dict(case=case, expected=expected, reference_objective=oracle.getObjectiveValue(), got=got)
                failures.append(failure)
                out = Path("results/lp-failures")
                out.mkdir(parents=True, exist_ok=True)
                (out / f"{args.seed}-{case}.mps").write_text(path.read_text())
                (out / f"{args.seed}-{case}.json").write_text(json.dumps(failure, indent=2) + "\n")
                if len(failures) >= 10:
                    break
    summary = dict(seed=args.seed, cases=case + 1, failures=len(failures), statuses=statuses,
                   unresolved=unresolved, oracle_ipm_fallbacks=oracle_fallbacks,
                   binary_sha256=binary_sha256)
    print(json.dumps(summary))
    for failure in failures:
        print(json.dumps(failure))
    raise SystemExit(bool(failures))


if __name__ == "__main__":
    main()
