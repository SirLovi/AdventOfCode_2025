import argparse
import logging
import os
import re
import shutil
import time
from pathlib import Path

import html2text
import requests
from bs4 import BeautifulSoup

# Fetches daily AoC pages/inputs and scaffolds solution folders.

##################################################################################################
# Configuration
##################################################################################################

DEFAULT_YEAR = 2025
DEFAULT_DELAY = 1.0
TEMPLATE_FILE = Path("AOC_TEMPLATE.py")
RUST_TEMPLATE_FILE = Path("AOC_TEMPLATE.rs")
DEFAULT_USER_AGENT = os.environ.get(
    "AOC_USER_AGENT",
    "github.com/your-handle/AdventOfCode_2025 (please set AOC_USER_AGENT with contact info)",
)

RUST_FALLBACK = """\
use anyhow::{anyhow, bail, Result};
use aoc2025::{confirm_prompt, detect_part, get_input, lines, load_example, submit_answer, time, DEFAULT_YEAR};

const DAY: u8 = {{DAY}};

fn part1(input: &str) -> Result<i64> {
    // TODO: implement real logic
    Ok(lines(input).count() as i64)
}

fn part2(input: &str) -> Result<i64> {
    // TODO: implement real logic
    Ok(input.lines().map(|l| l.len() as i64).sum())
}

#[derive(Debug, Default)]
struct Args {
    part: Option<u8>,
    year: i32,
    example: bool,
    submit: bool,
    no_confirm: bool,
}

fn parse_args() -> Result<Args> {
    let mut args = Args {
        year: DEFAULT_YEAR,
        ..Default::default()
    };

    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--part" => {
                let val = iter
                    .next()
                    .ok_or_else(|| anyhow!("--part requires a value"))?;
                args.part = Some(val.parse()?);
            }
            "--year" => {
                let val = iter
                    .next()
                    .ok_or_else(|| anyhow!("--year requires a value"))?;
                args.year = val.parse()?;
            }
            "--example" => args.example = true,
            "--submit" => args.submit = true,
            "--no-confirm" => args.no_confirm = true,
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => bail!("Unknown argument: {other}"),
        }
    }

    Ok(args)
}

fn print_usage() {
    eprintln!(
        "\
Day {day} runner
  --part <1|2>     Force part (default: detect instructions-two.md)
  --year <YYYY>    Override year (default: {})
  --example        Use Example_{day_pad}.txt if present
  --submit         Submit the computed answer
  --no-confirm     Skip prompt when submitting
",
        DEFAULT_YEAR
    );
}

fn main() -> Result<()> {
    let args = parse_args()?;
    let part = args.part.unwrap_or_else(|| detect_part(DAY));

    let raw = if args.example {
        load_example(DAY)?
    } else {
        get_input(DAY, args.year)?
    };

    let (ans1, t1) = time(|| part1(&raw).unwrap());
    println!("Part 1: {ans1} ({t1} ms)");

    let (ans2, t2) = time(|| part2(&raw).unwrap());
    println!("Part 2: {ans2} ({t2} ms)");

    if args.submit {
        let answer = match part {
            1 => ans1,
            2 => ans2,
            _ => bail!("Part must be 1 or 2"),
        };

        if !args.no_confirm {
            confirm_prompt()?;
        }

        let verdict = submit_answer(DAY, part, answer, args.year)?;
        println!("Submission verdict: {verdict}");
    }

    Ok(())
}
"""

logging.basicConfig(
    filename="aoc_fetch.log",
    level=logging.INFO,
    format="%(asctime)s %(levelname)s: %(message)s",
)
logger = logging.getLogger(__name__)
logger.addHandler(logging.StreamHandler())


def load_session(explicit: str | None = None) -> str:
    candidates = [
        explicit,
        os.environ.get("AOC_SESSION_ID"),
        Path("SessionID.txt"),
    ]
    for candidate in candidates:
        if isinstance(candidate, str) and candidate.strip():
            return candidate.strip()
        if isinstance(candidate, Path) and candidate.exists():
            return candidate.read_text().strip()
    raise SystemExit(
        "Missing session cookie. Set AOC_SESSION_ID or create SessionID.txt"
    )


def build_session(session_id: str) -> requests.Session:
    sess = requests.Session()
    sess.headers.update(
        {"cookie": f"session={session_id}", "User-Agent": DEFAULT_USER_AGENT}
    )
    return sess


def save_markdown(article, path: Path) -> None:
    h = html2text.HTML2Text()
    h.body_width = 0
    path.write_text(h.handle(str(article)).strip("\n"))


def save_input(text: str, day_dir: Path, day: int) -> None:
    (day_dir / f"input_{day:02d}.txt").write_text(text.rstrip("\n"))


def extract_example(soup: BeautifulSoup) -> str | None:
    block = soup.select_one("article pre code")
    if block:
        return block.get_text("\n").rstrip("\n")
    return None


def precreate_day(
    day: int,
    *,
    copy_template: bool,
    force_template: bool,
    scaffold_rust: bool,
    cargo_toml: Path,
    rust_template: Path,
) -> None:
    """Create day folder and drop templates even before release."""

    day_dir = Path(f"Day_{day:02d}")
    day_dir.mkdir(parents=True, exist_ok=True)

    if copy_template:
        copy_python_template(day, day_dir, force_template)

    if scaffold_rust:
        scaffold_rust_bin(day, day_dir, cargo_toml, rust_template)


def copy_python_template(day: int, day_dir: Path, force_template: bool) -> None:
    solution_path = day_dir / f"Solution_{day:02d}.py"
    if solution_path.exists() and not force_template:
        return

    if TEMPLATE_FILE.exists():
        shutil.copyfile(TEMPLATE_FILE, solution_path)
    else:
        logger.warning(
            f"Template {TEMPLATE_FILE} not found; skipping copy for Day {day}"
        )


def scaffold_rust_bin(day: int, day_dir: Path, cargo_toml: Path, rust_template: Path):
    bin_path = day_dir / f"day{day:02d}.rs"
    if not bin_path.exists():
        contents = (
            rust_template.read_text() if rust_template.exists() else RUST_FALLBACK
        )
        contents = contents.replace("{{DAY}}", str(day)).replace(
            "{{DAY_PAD}}", f"{day:02d}"
        )
        bin_path.write_text(contents)
        logger.info(f"Created Rust bin {bin_path}")
    register_bin_in_cargo(day, cargo_toml)


def register_bin_in_cargo(day: int, cargo_toml: Path) -> None:
    name = f"day{day:02d}"
    if not cargo_toml.exists():
        logger.warning(f"Cargo.toml not found; cannot register bin {name}")
        return

    text = cargo_toml.read_text()
    if re.search(rf'name\s*=\s*"{re.escape(name)}"', text):
        return

    block = "\n[[bin]]\n" f'name = "{name}"\n' f'path = "Day_{day:02d}/{name}.rs"\n'
    cargo_toml.write_text(text.rstrip() + block + "\n")
    logger.info(f"Registered bin {name} in Cargo.toml")


def process_day(
    day: int,
    *,
    year: int,
    session: requests.Session,
    copy_template: bool,
    force_template: bool,
    scaffold_rust: bool,
    cargo_toml: Path,
    rust_template: Path,
) -> bool:
    puzzle_url = f"https://adventofcode.com/{year}/day/{day}"
    logger.info(f"Day {day}: fetching {puzzle_url}")
    puzzle_response = session.get(puzzle_url)

    if puzzle_response.status_code == 404:
        logger.info(f"Day {day}: not released yet (404)")
        precreate_day(
            day,
            copy_template=copy_template,
            force_template=force_template,
            scaffold_rust=scaffold_rust,
            cargo_toml=cargo_toml,
            rust_template=rust_template,
        )
        return False
    if puzzle_response.status_code != 200:
        logger.warning(f"Day {day}: HTTP {puzzle_response.status_code}, skipping")
        return True

    soup = BeautifulSoup(puzzle_response.text, "html.parser")
    articles = soup.find_all("article")
    if not articles:
        logger.info(f"Day {day}: page has no articles yet, stopping")
        return False

    day_dir = Path(f"Day_{day:02d}")
    day_dir.mkdir(parents=True, exist_ok=True)

    # Save instructions
    save_markdown(articles[0], day_dir / "instructions-one.md")
    if len(articles) > 1:
        save_markdown(articles[1], day_dir / "instructions-two.md")

    # Save example (first code block)
    example = extract_example(soup)
    if example:
        (day_dir / f"Example_{day:02d}.txt").write_text(example)

    # Fetch input
    input_url = f"https://adventofcode.com/{year}/day/{day}/input"
    input_response = session.get(input_url)
    if input_response.status_code == 200:
        save_input(input_response.text, day_dir, day)
    else:
        logger.warning(
            f"Day {day}: input unavailable (HTTP {input_response.status_code})"
        )

    # Copy template
    if copy_template:
        copy_python_template(day, day_dir, force_template)

    if scaffold_rust:
        scaffold_rust_bin(day, day_dir, cargo_toml, rust_template)

    logger.info(f"Day {day}: done")
    return True


def main():
    parser = argparse.ArgumentParser(
        description="Fetch AoC pages and scaffold day folders"
    )
    parser.add_argument(
        "--year",
        type=int,
        default=DEFAULT_YEAR,
        help="Year to fetch (default: %(default)s)",
    )
    parser.add_argument(
        "--start-day", type=int, default=1, help="First day to attempt (default: 1)"
    )
    parser.add_argument(
        "--end-day", type=int, default=25, help="Last day to attempt (default: 25)"
    )
    parser.add_argument(
        "--delay",
        type=float,
        default=DEFAULT_DELAY,
        help="Seconds to sleep between days (default: %(default)s)",
    )
    parser.add_argument(
        "--skip-template",
        action="store_true",
        help="Do not copy AOC_TEMPLATE.py into day folders",
    )
    parser.add_argument(
        "--force-template",
        action="store_true",
        help="Overwrite existing Solution_XX.py",
    )
    parser.add_argument(
        "--no-rust",
        action="store_true",
        help="Skip Rust bin scaffolding / Cargo registration",
    )
    parser.add_argument(
        "--rust-template",
        type=str,
        default=str(RUST_TEMPLATE_FILE),
        help="Path to Rust template file (default: AOC_TEMPLATE.rs)",
    )
    parser.add_argument("--session", type=str, help="Explicit session cookie value")
    args = parser.parse_args()

    session_id = load_session(args.session)
    os.environ["AOC_SESSION_ID"] = session_id
    session = build_session(session_id)

    print("Starting to fetch Advent of Code data...")
    cargo_toml = Path("Cargo.toml")
    rust_template = Path(args.rust_template)

    for day in range(args.start_day, args.end_day + 1):
        ok = process_day(
            day,
            year=args.year,
            session=session,
            copy_template=not args.skip_template,
            force_template=args.force_template,
            scaffold_rust=not args.no_rust,
            cargo_toml=cargo_toml,
            rust_template=rust_template,
        )
        if not ok:
            print(f"Day {day} not available yet; stopping.")
            break
        time.sleep(max(args.delay, 0))

    print("Done fetching Advent of Code data.")


if __name__ == "__main__":
    main()
