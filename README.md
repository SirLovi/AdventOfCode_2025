# Advent of Code 2025

Small scaffold to fetch each day's puzzle, cache inputs, and keep Python + Rust solutions in sync.

## What you get
- `RUN_EVERY_DAY.py` fetcher: saves puzzle pages as `instructions-one.md`/`instructions-two.md`, grabs the first example block as `Example_XX.txt`, fetches inputs, and copies `AOC_TEMPLATE.py` into `Day_XX/Solution_XX.py` (skips existing files unless `--force-template`).
- Python runner template (`Day_XX/Solution_XX.py`): auto-detects part from `instructions-two.md`, supports `--example`, optional submission with confirm prompt, and caches inputs as `input.txt` + `input_XX.txt`.
- Rust helper crate `aoc2025` with per-day bins (`Day_XX/dayXX.rs`) sharing the same cache/session cookie; helper fns live in `src/lib.rs`.
- `scripts/new_day.sh` to scaffold a new Rust bin and register it in `Cargo.toml`.

## Setup once
- Put your AoC session cookie in `AOC_SESSION_ID` or a `SessionID.txt` at repo root (a day-local `Day_XX/SessionID.txt` also works).
- Set a polite user agent: `export AOC_USER_AGENT="github.com/<you>/AdventOfCode_2025 (email@example.com)"`.
- Python deps: `python -m venv .venv && source .venv/bin/activate && pip install -r requirements.txt`.
- Rust toolchain: install Rust + components `rustup component add clippy rustfmt`.

## Daily flow - Python
1. Fetch unlocked days (stops at first 404):
   ```bash
   python RUN_EVERY_DAY.py --year 2025
   # flags: --start-day, --end-day, --delay, --skip-template, --force-template, --session
   ```
   Creates `Day_XX` with instructions, example, inputs, and a copied template even if the day isn't released yet.
2. Solve/run a day:
   ```bash
   python Day_01/Solution_01.py --part 1      # force part 1
   python Day_01/Solution_01.py --part 2      # force part 2
   python Day_01/Solution_01.py --example     # run against Example_01.txt
   python Day_01/Solution_01.py --submit      # submit (prompts unless --no-confirm)
   ```
   Env override: `AOC_YEAR` or `--year` to run against a different year.

## Daily flow - Rust
- (Optional) scaffold a bin: `scripts/new_day.sh 02` -> `Day_02/day02.rs` plus `Cargo.toml` entry (includes the same CLI flags as `day01`: `--part`, `--example`, `--submit`, `--no-confirm`, `--year`).
- Fetch inputs via the Python fetcher **or** let the Rust helpers grab/cache them on first run (`get_input` reads `AOC_SESSION_ID`/`SessionID.txt`).
- Run a day:
  ```bash
  cargo run --bin day01                       # prints both parts
  cargo run --bin day01 -- --example          # use Example_01.txt if present
  cargo run --bin day01 -- --part 2 --submit  # submit chosen part (add --no-confirm to skip prompt)
  ```
- Keep things tidy: `cargo fmt && cargo clippy --all-targets`.
- Shared utilities (timing, input cache, submission helper) are in `src/lib.rs`.

## Notes
- Puzzle inputs (`Day_*/input*.txt`) stay gitignored.
- Cache/user-agent/session handling is shared between Python and Rust; set them once and both tracks work.
- Consider a daily cron/systemd timer around 05:05 UTC (midnight US Eastern) to run `RUN_EVERY_DAY.py` automatically.
