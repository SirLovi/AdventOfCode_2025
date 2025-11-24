use anyhow::{bail, Result};
use aoc2025::{
    confirm_prompt, detect_part, get_input, lines, load_example, submit_answer, time, DEFAULT_YEAR,
};
use std::env;

fn part1(input: &str) -> Result<i64> {
    let count = lines(input).count() as i64;
    Ok(count)
}

fn part2(input: &str) -> Result<i64> {
    let total: i64 = input.lines().map(|l| l.len() as i64).sum();
    Ok(total)
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
Day 01 runner
  --part <1|2>     Force part (default: detect instructions-two.md)
  --year <YYYY>    Override year (default: {})
  --example        Use Example_01.txt if present
  --submit         Submit the computed answer
  --no-confirm     Skip prompt when submitting
",
        DEFAULT_YEAR
    );
}

fn main() -> Result<()> {
    let args = parse_args()?;
    let part = args.part.unwrap_or_else(|| detect_part(1));

    let raw = if args.example {
        load_example(1)?
    } else {
        get_input(1, args.year)?
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

        let verdict = submit_answer(1, part, answer, args.year)?;
        println!("Submission verdict: {verdict}");
    }

    Ok(())
}
