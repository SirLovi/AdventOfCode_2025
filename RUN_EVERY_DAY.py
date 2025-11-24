import argparse
import logging
import os
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
DEFAULT_USER_AGENT = os.environ.get(
    "AOC_USER_AGENT",
    "github.com/your-handle/AdventOfCode_2025 (please set AOC_USER_AGENT with contact info)",
)

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
    for filename in ["input.txt", f"input_{day:02d}.txt"]:
        (day_dir / filename).write_text(text.rstrip("\n"))


def extract_example(soup: BeautifulSoup) -> str | None:
    block = soup.select_one("article pre code")
    if block:
        return block.get_text("\n").rstrip("\n")
    return None


def precreate_day(day: int, *, copy_template: bool, force_template: bool) -> None:
    """Create day folder and drop template even before release."""

    day_dir = Path(f"Day_{day:02d}")
    day_dir.mkdir(parents=True, exist_ok=True)

    if not copy_template:
        return

    solution_path = day_dir / f"Solution_{day:02d}.py"
    if solution_path.exists() and not force_template:
        return

    if TEMPLATE_FILE.exists():
        shutil.copyfile(TEMPLATE_FILE, solution_path)
    else:
        logger.warning(
            f"Template {TEMPLATE_FILE} not found; skipping copy for Day {day}"
        )


def process_day(
    day: int,
    *,
    year: int,
    session: requests.Session,
    copy_template: bool,
    force_template: bool,
) -> bool:
    puzzle_url = f"https://adventofcode.com/{year}/day/{day}"
    logger.info(f"Day {day}: fetching {puzzle_url}")
    puzzle_response = session.get(puzzle_url)

    if puzzle_response.status_code == 404:
        logger.info(f"Day {day}: not released yet (404)")
        precreate_day(day, copy_template=copy_template, force_template=force_template)
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
    solution_path = day_dir / f"Solution_{day:02d}.py"
    if copy_template:
        if solution_path.exists() and not force_template:
            logger.info(f"Day {day}: Solution file exists, skipping template copy")
        elif TEMPLATE_FILE.exists():
            shutil.copyfile(TEMPLATE_FILE, solution_path)
        else:
            logger.warning(f"Template {TEMPLATE_FILE} not found; skipping copy")

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
    parser.add_argument("--session", type=str, help="Explicit session cookie value")
    args = parser.parse_args()

    session_id = load_session(args.session)
    os.environ["AOC_SESSION_ID"] = session_id  # Keep consistent for spawned scripts
    session = build_session(session_id)

    print("Starting to fetch Advent of Code data...")
    for day in range(args.start_day, args.end_day + 1):
        ok = process_day(
            day,
            year=args.year,
            session=session,
            copy_template=not args.skip_template,
            force_template=args.force_template,
        )
        if not ok:
            print(f"Day {day} not available yet; stopping.")
            break
        time.sleep(max(args.delay, 0))

    print("Done fetching Advent of Code data.")


if __name__ == "__main__":
    main()
