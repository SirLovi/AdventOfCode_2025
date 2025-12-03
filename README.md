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
# Additional options:
# --start-day 1 //
# --end-day 24 //
# --delay //
# --skip-template //
# --force-template //
# --no-rust //
# --rust-template AOC_TEMPLATE.rs //
# --session <cookie>
```
Creates `Day_XX` with instructions, example, `input_XX.txt`, copies `AOC_TEMPLATE.py`, and scaffolds `dayXX.rs` (registers in `Cargo.toml`).

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
cargo run --bin day01 -- --part 2 --submit   # add --no-confirm to skip prompt
```
Shared helpers (input cache, part detect, submit, timing) live in `src/lib.rs`. Inputs cache to `Day_XX/input_XX.txt` (legacy `input.txt` still read if present).
