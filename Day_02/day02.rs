use anyhow::{anyhow, bail, Result};
use aoc2025::{
    confirm_prompt, detect_part, get_input, load_example, submit_answer, time_result, uints,
    DEFAULT_YEAR,
};
use std::convert::TryFrom;

const DAY: u8 = 2;

//##################################################################################################
// Parsing & Data Prep & Puzzle Logic
//##################################################################################################

type Range = (u64, u64);

fn parse_ranges(input: &str) -> Result<Vec<Range>> {
    let nums = uints(input);
    if nums.is_empty() {
        bail!("No ranges parsed from input");
    }
    if nums.len() % 2 != 0 {
        bail!("Odd number of endpoints in input; expected start/end pairs");
    }

    let mut ranges = Vec::with_capacity(nums.len() / 2);
    for chunk in nums.chunks_exact(2) {
        let (start, end) = (chunk[0], chunk[1]);
        if start > end {
            bail!("Range start > end: {start}-{end}");
        }
        ranges.push((start, end));
    }

    Ok(ranges)
}

fn merge_ranges(mut ranges: Vec<Range>) -> Vec<Range> {
    if ranges.is_empty() {
        return ranges;
    }

    ranges.sort_by_key(|&(a, _)| a);
    let mut merged = Vec::with_capacity(ranges.len());
    let mut cur = ranges[0];

    for (a, b) in ranges.into_iter().skip(1) {
        if a <= cur.1 + 1 {
            cur.1 = cur.1.max(b);
        } else {
            merged.push(cur);
            cur = (a, b);
        }
    }

    merged.push(cur);
    merged
}

fn contains(ranges: &[(u64, u64)], x: u64) -> bool {
    let idx = ranges
        .binary_search_by(|&(start, _)| start.cmp(&x))
        .unwrap_or_else(|i| i);

    if idx < ranges.len() && ranges[idx].0 == x {
        return true;
    }

    if idx == 0 {
        return false;
    }

    let (_, end) = ranges[idx - 1];
    x <= end
}

fn sum_repeated_pairs(ranges: &[Range]) -> i128 {
    let max_val = ranges.iter().map(|&(_, b)| b).max().unwrap_or(0) as u128;
    let mut total: i128 = 0;

    let mut pow10: u128 = 10;
    loop {
        let prefix_min = pow10 / 10;
        let base = pow10 + 1;
        let smallest = prefix_min * base;

        if smallest > max_val {
            break;
        }

        let prefix_max = pow10 - 1;
        for prefix in prefix_min..=prefix_max {
            let n = prefix * base;
            if n > max_val {
                break;
            }

            let n_u64 = n as u64;
            if contains(ranges, n_u64) {
                total += n as i128;
            }
        }

        pow10 = match pow10.checked_mul(10) {
            Some(v) => v,
            None => break,
        };
    }

    total
}

fn num_digits(mut n: u64) -> usize {
    if n == 0 {
        return 1;
    }
    let mut d = 0;
    while n > 0 {
        d += 1;
        n /= 10;
    }
    d
}

fn pow10_table(max_digits: usize) -> Vec<u128> {
    let mut v = Vec::with_capacity(max_digits + 2);
    let mut cur: u128 = 1;
    v.push(cur);
    for _ in 0..=max_digits {
        cur *= 10;
        v.push(cur);
    }
    v
}

fn repeat_num(prefix: u128, base: u128, times: usize) -> u128 {
    let mut n = 0u128;
    for _ in 0..times {
        n = n * base + prefix;
    }
    n
}

fn sum_repeated_at_least_twice(ranges: &[Range]) -> i128 {
    use std::collections::HashSet;

    let max_end = ranges.iter().map(|&(_, b)| b).max().unwrap_or(0);
    if max_end == 0 {
        return 0;
    }

    let max_digits = num_digits(max_end);
    let pow10 = pow10_table(max_digits);
    let mut seen = HashSet::new();
    let mut total: i128 = 0;

    for block_len in 1..=max_digits {
        let base = pow10[block_len] as u128;
        let prefix_min = pow10[block_len - 1] as u128;
        let prefix_max = base - 1;

        let max_repeat = max_digits / block_len;
        for k in 2..=max_repeat {
            let smallest = repeat_num(prefix_min, base, k);
            if smallest > max_end as u128 {
                break;
            }

            let mut prefix = prefix_min;
            while prefix <= prefix_max {
                let n = repeat_num(prefix, base, k);
                if n > max_end as u128 {
                    break;
                }

                let n64 = n as u64;
                if contains(ranges, n64) && seen.insert(n64) {
                    total += n as i128;
                }
                prefix += 1;
            }
        }
    }

    total
}

//##################################################################################################
// Solutions
//##################################################################################################

fn part1(input: &str) -> Result<i64> {
    let ranges = merge_ranges(parse_ranges(input)?);
    let sum = sum_repeated_pairs(&ranges);
    let ans = i64::try_from(sum).map_err(|_| anyhow!("part1 sum exceeds i64"))?;
    Ok(ans)
}

fn part2(input: &str) -> Result<i64> {
    let ranges = merge_ranges(parse_ranges(input)?);
    let sum = sum_repeated_at_least_twice(&ranges);
    let ans = i64::try_from(sum).map_err(|_| anyhow!("part2 sum exceeds i64"))?;
    Ok(ans)
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
        day_pad = "02",
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
