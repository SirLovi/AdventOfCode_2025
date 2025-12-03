use anyhow::{bail, Result};
use aoc2025::{
    confirm_prompt, detect_part, get_input, lines, load_example, submit_answer, time_result,
    DEFAULT_YEAR,
};
use std::env;

const DAY: u8 = 1;

//##################################################################################################
// Parsing & Data Prep & Puzzle Logic
//##################################################################################################

fn parse(input: &str) -> Result<Vec<(char, i64)>> {
    let mut res = Vec::new();
    for line in lines(input).filter(|l| !l.is_empty()) {
        let (dch, num) = line.split_at(1);
        let dir = match dch.chars().next() {
            Some(c @ ('L' | 'R')) => c,
            _ => bail!("Invalid direction in line: {line}"),
        };
        let dist: i64 = num.parse()?;
        res.push((dir, dist));
    }
    Ok(res)
}

fn zero_hits(pos: i64, dir: char, steps: i64) -> i64 {
    let m = 100i64;
    let pos = pos.rem_euclid(m);
    let mut hits = if pos == 0 { 1 } else { 0 };

    if steps == 0 {
        return hits;
    }

    let first = match dir {
        'R' => (m - pos) % m,
        'L' => pos % m,
        _ => unreachable!(),
    };

    let first = if first == 0 { m } else { first };

    if steps >= first {
        hits += 1 + (steps - first) / m;
    }

    hits
}

//##################################################################################################
// Solutions
//##################################################################################################

fn part1(input: &str) -> Result<i64> {
    let mut pos: i64 = 50;
    let mut zeros = 0;

    for (dir, dist) in parse(input)? {
        let delta = if dir == 'R' { dist } else { -dist };
        pos = (pos + delta).rem_euclid(100);
        if pos == 0 {
            zeros += 1;
        }
    }

    Ok(zeros)
}

fn part2(input: &str) -> Result<i64> {
    let mut pos: i64 = 50;
    let mut zeros = 0;

    for (dir, dist) in parse(input)? {
        zeros += zero_hits(pos, dir, dist);

        let delta = if dir == 'R' { dist } else { -dist };
        pos = (pos + delta).rem_euclid(100);
    }

    Ok(zeros)
}

//##################################################################################################
// CLI Arguments
//##################################################################################################

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

    let mut iter = env::args().skip(1);
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
Day {day} runner
  --part <1|2>     Force part (default: detect instructions-two.md)
  --year <YYYY>    Override year (default: {default_year})
  --example        Use Example_{day_pad}.txt if present
  --submit         Submit the computed answer
  --no-confirm     Skip prompt when submitting
",
        day = DAY,
        day_pad = "01",
        default_year = DEFAULT_YEAR
    );
}

//##################################################################################################
// Entry Point
//##################################################################################################

fn main() -> Result<()> {
    let args = parse_args()?;
    let part = args.part.unwrap_or_else(|| detect_part(DAY));

    let raw = if args.example {
        load_example(DAY)?
    } else {
        get_input(DAY, args.year)?
    };

    let (ans1, t1) = time_result(|| part1(&raw))?;
    println!("Part 1: {ans1} ({t1} ms)");

    let (ans2, t2) = time_result(|| part2(&raw))?;
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
