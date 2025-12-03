use anyhow::{anyhow, bail, Result};
use aoc2025::{
    confirm_prompt, detect_part, get_input, lines, load_example, submit_answer, time_result,
    DEFAULT_YEAR,
};

const DAY: u8 = {
    {
        DAY
    }
};

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
  --year <YYYY>    Override year (default: {default_year})
  --example        Use Example_{day_pad}.txt if present
  --submit         Submit the computed answer
  --no-confirm     Skip prompt when submitting
",
        day = DAY,
        day_pad = "{{DAY_PAD}}",
        default_year = DEFAULT_YEAR
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
