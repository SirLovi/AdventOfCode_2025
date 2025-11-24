#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "Usage: scripts/new_day.sh <day-number>" >&2
  exit 1
fi

DAY_NUM=$1
DAY=$(printf "%02d" "$DAY_NUM")
ROOT=$(cd "$(dirname "$0")/.." && pwd)
DAY_DIR="$ROOT/Day_${DAY}"
BIN_PATH="$DAY_DIR/day${DAY}.rs"

mkdir -p "$DAY_DIR"

if [[ ! -f "$BIN_PATH" ]]; then
  cat > "$BIN_PATH" <<EOR
use anyhow::{bail, Result};
use aoc2025::{
    confirm_prompt, detect_part, get_input, lines, load_example, submit_answer, time, DEFAULT_YEAR,
};

const DAY: u8 = ${DAY_NUM};

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
                    .ok_or_else(|| anyhow::anyhow!("--part requires a value"))?;
                args.part = Some(val.parse()?);
            }
            "--year" => {
                let val = iter
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--year requires a value"))?;
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
Day {DAY} runner
  --part <1|2>     Force part (default: detect instructions-two.md)
  --year <YYYY>    Override year (default: {})
  --example        Use Example_{DAY:02}.txt if present
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
EOR
  echo "Created $BIN_PATH"
else
  echo "Bin already exists: $BIN_PATH"
fi

# Add [[bin]] entry if missing
if ! grep -q "name = \"day${DAY}\"" "$ROOT/Cargo.toml"; then
  cat >> "$ROOT/Cargo.toml" <<EOT
[[bin]]
name = "day${DAY}"
path = "Day_${DAY}/day${DAY}.rs"
EOT
  echo "Registered bin day${DAY} in Cargo.toml"
else
  echo "Bin day${DAY} already registered in Cargo.toml"
fi
