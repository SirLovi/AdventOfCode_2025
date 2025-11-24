"""
Minimal AoC solution harness to be copied per day.

Features:
- Reads session cookie from env `AOC_SESSION_ID` or `SessionID.txt` in the day folder or repo root.
- Fetches and caches puzzle input as `input.txt` (and `input_XX.txt` for compatibility).
- Detects the current day from the directory name (e.g. Day_04) and picks part 2 automatically when
  `instructions-two.md` exists.
- Optional submission with prompt to avoid accidental posts.
"""

from __future__ import annotations

import argparse
import logging
import os
import re
import sys
from pathlib import Path

import requests

##################################################################################################
# Configuration helpers
##################################################################################################

BASE_DIR = Path(__file__).parent.resolve()

day_match = re.match(r"Day_(\d+)$", BASE_DIR.name)
if not day_match:
    raise SystemExit(
        "This template is meant to be copied into a Day_XX folder. Run the per-day copy (e.g. Day_01/Solution_01.py)."
    )

DAY = int(day_match.group(1))
DEFAULT_YEAR = 2025
YEAR = int(os.environ.get("AOC_YEAR", DEFAULT_YEAR))

USER_AGENT = os.environ.get(
    "AOC_USER_AGENT",
    "github.com/your-handle/AdventOfCode_2025 (please set AOC_USER_AGENT with contact info)",
)

LOG_FILE = BASE_DIR / "aoc_solution.log"
logging.basicConfig(
    filename=LOG_FILE,
    level=logging.INFO,
    format="%(asctime)s %(levelname)s: %(message)s",
)
logger = logging.getLogger(__name__)
logger.addHandler(logging.StreamHandler(sys.stdout))


def load_session() -> str:
    """Return session id from env or SessionID.txt (day folder first, then repo root)."""

    candidates = [
        os.environ.get("AOC_SESSION_ID"),
        BASE_DIR / "SessionID.txt",
        BASE_DIR.parent / "SessionID.txt",
    ]

    for candidate in candidates:
        if isinstance(candidate, str) and candidate.strip():
            return candidate.strip()
        if isinstance(candidate, Path) and candidate.exists():
            return candidate.read_text().strip()

    raise RuntimeError(
        "Missing session cookie. Set AOC_SESSION_ID or place SessionID.txt next to this file or at repo root."
    )


SESSION_ID = load_session()
session = requests.Session()
session.headers.update({"cookie": f"session={SESSION_ID}", "User-Agent": USER_AGENT})

INPUT_CANDIDATES = [
    BASE_DIR / "input.txt",
    BASE_DIR / f"input_{DAY:02d}.txt",
    BASE_DIR / f"input_{DAY}.txt",
]


def detect_part() -> int:
    return 2 if (BASE_DIR / "instructions-two.md").exists() else 1


def read_cached_input() -> str | None:
    for candidate in INPUT_CANDIDATES:
        if candidate.exists():
            logger.info(f"Using cached input: {candidate}")
            return candidate.read_text()
    return None


def cache_input(contents: str) -> None:
    primary = BASE_DIR / "input.txt"
    primary.write_text(contents)

    alt = BASE_DIR / f"input_{DAY:02d}.txt"
    if alt != primary:
        alt.write_text(contents)


def fetch_input(day: int, year: int) -> str:
    url = f"https://adventofcode.com/{year}/day/{day}/input"
    logger.info(f"Fetching input from {url}")
    response = session.get(url)
    if response.status_code != 200:
        raise RuntimeError(
            f"Failed to fetch input (HTTP {response.status_code}). Check session cookie and year/day."
        )
    return response.text


def get_input(day: int = DAY, year: int = YEAR) -> str:
    cached = read_cached_input()
    if cached is not None:
        return cached

    contents = fetch_input(day, year)
    cache_input(contents)
    return contents


def submit(
    day: int, level: int, answer: str | int, year: int = YEAR, confirm: bool = True
) -> str:
    print(f"\nPreparing to submit Year {year}, Day {day}, Level {level}: {answer}")
    if confirm:
        try:
            input("Press Enter to submit or Ctrl+C to abort... ")
        except KeyboardInterrupt:
            raise SystemExit("Submission cancelled.")

    url = f"https://adventofcode.com/{year}/day/{day}/answer"
    data = {"level": str(level), "answer": str(answer)}
    logger.info(f"Submitting answer for day {day}, level {level}")
    response = session.post(url, data=data)
    text = response.text

    if "You gave an answer too recently" in text:
        verdict = "TOO MANY REQUESTS"
    elif "not the right answer" in text:
        if "too low" in text:
            verdict = "WRONG (TOO LOW)"
        elif "too high" in text:
            verdict = "WRONG (TOO HIGH)"
        else:
            verdict = "WRONG"
    elif "You don't seem to be solving the right level." in text:
        verdict = "ALREADY SOLVED"
    else:
        verdict = "OK"

    logger.info(f"Submission verdict: {verdict}")
    print(f"VERDICT: {verdict}")
    return verdict


##################################################################################################
# Solution stubs (replace with actual logic)
##################################################################################################


def solve_part1(puzzle_input: str):
    lines = puzzle_input.strip().split("\n")
    return len(lines)


def solve_part2(puzzle_input: str):
    lines = puzzle_input.strip().split("\n")
    return sum(len(line) for line in lines)


##################################################################################################
# CLI
##################################################################################################


def parse_args():
    parser = argparse.ArgumentParser(description="AoC daily runner")
    parser.add_argument(
        "--part",
        type=int,
        choices=[1, 2],
        help="Force part (default: detect by instructions-two.md)",
    )
    parser.add_argument(
        "--year", type=int, default=YEAR, help="Override year (default: %(default)s)"
    )
    parser.add_argument(
        "--example",
        action="store_true",
        help="Use Example_XX.txt if present instead of input",
    )
    parser.add_argument(
        "--submit", action="store_true", help="Submit the computed answer"
    )
    parser.add_argument(
        "--no-confirm", action="store_true", help="Skip prompt when --submit is set"
    )
    return parser.parse_args()


def load_example() -> str:
    candidates = [
        BASE_DIR / f"Example_{DAY:02d}.txt",
        BASE_DIR / "example.txt",
    ]
    for path in candidates:
        if path.exists():
            logger.info(f"Using example input: {path}")
            return path.read_text()
    raise FileNotFoundError("No example file found.")


def main() -> None:
    args = parse_args()
    part = args.part or detect_part()
    year = args.year

    if args.example:
        puzzle_input = load_example().strip("\n")
    else:
        puzzle_input = get_input(DAY, year).strip("\n")

    if part == 1:
        answer = solve_part1(puzzle_input)
    else:
        answer = solve_part2(puzzle_input)

    print(f"Answer (part {part}): {answer}")

    if args.submit:
        submit(DAY, part, answer, year=year, confirm=not args.no_confirm)


if __name__ == "__main__":
    try:
        main()
    except Exception as exc:  # keep simple for contest speed
        logger.error(f"Error: {exc}")
        raise
