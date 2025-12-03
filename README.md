# Advent of Code 2025

Scaffold to fetch puzzles and run Python/Rust solutions with shared helpers for [Advent of Code](https://adventofcode.com)

## Setup
- Session cookie: set `AOC_SESSION_ID` or place `SessionID.txt` at repo root (day-local `Day_XX/SessionID.txt` also works).
- User agent: `export AOC_USER_AGENT="github.com/<you>/AdventOfCode_2025 (email@example.com)"`.
- Python deps: `python -m venv .venv && source .venv/bin/activate && pip install -r requirements.txt`.
- Rust: `rustup component add clippy rustfmt`.

## Fetch everything
```bash
python RUN_EVERY_DAY.py --year 2025
```
Creates `Day_XX` with instructions, example, `input_XX.txt`, copies `AOC_TEMPLATE.py`, and scaffolds `dayXX.rs` (registers in `Cargo.toml`).

Additional options:
- `--start-day 1`
- `--end-day 24`
- `--delay`
- `--skip-template`
- `--force-template`
- `--no-rust`
- `--rust-template AOC_TEMPLATE.rs`
- `--session <cookie>`

## Run Python
```bash
python Day_01/Solution_01.py --part 1        # force part
python Day_01/Solution_01.py --example       # use Example_01.txt
python Day_01/Solution_01.py --submit        # add --no-confirm to skip prompt
```
`AOC_YEAR` or `--year` overrides the target year.

## Run Rust
```bash
cargo run --bin day01                        # prints both parts
cargo run --bin day01 -- --example           # use example
cargo run --bin day01 -- --part 1 --submit   # add --no-confirm to skip prompt
```
Shared helpers live in `src/lib.rs` (input cache, part detect, submit, timing). Extras you can lean on:
- `time_result` to time fallible work without `unwrap`.
- `ints` / `uints` extract numbers from messy text.
- `Point`, `Dir4`, `in_bounds`, `neighbors4/8` for grid work.
- `counts`, `bfs_distances`, `dijkstra` for quick graph tasks.
- `gcd` / `lcm`, `digits`, `transpose` for common puzzle math.
Inputs cache to `Day_XX/input_XX.txt` (legacy `input.txt` still read if present).
