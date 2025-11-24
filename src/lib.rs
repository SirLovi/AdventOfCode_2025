use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;

pub const DEFAULT_YEAR: i32 = 2025;
const USER_AGENT_FALLBACK: &str =
    "github.com/your-handle/AdventOfCode_2025 (please set AOC_USER_AGENT with contact info)";

/// Load the puzzle input for the given day. If not cached locally, fetch from AoC and cache.
pub fn read_input(day: u8) -> Result<String> {
    get_input(day, DEFAULT_YEAR)
}

/// Fetch (or read cached) puzzle input for a given day/year.
pub fn get_input(day: u8, year: i32) -> Result<String> {
    if let Some(cached) = read_cached_input(day) {
        return Ok(cached);
    }
    let session = load_session(Some(day))?;
    let user_agent = load_user_agent();
    let client = http_client(&user_agent)?;
    let url = format!("https://adventofcode.com/{year}/day/{day}/input");
    let resp = client
        .get(url)
        .header("Cookie", format!("session={session}"))
        .send()
        .context("Failed to fetch puzzle input")?;

    if !resp.status().is_success() {
        return Err(anyhow!("HTTP {} when fetching input", resp.status()));
    }

    let body = resp.text().context("Reading input body")?;
    cache_input(day, &body)?;
    Ok(body)
}

fn read_cached_input(day: u8) -> Option<String> {
    input_candidates(day)
        .into_iter()
        .find_map(|path| fs::read_to_string(path).ok())
}

fn cache_input(day: u8, contents: &str) -> Result<()> {
    for path in input_candidates(day) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, contents)
            .with_context(|| format!("Failed to write input cache: {}", path.display()))?;
    }
    Ok(())
}

fn input_candidates(day: u8) -> Vec<PathBuf> {
    vec![
        PathBuf::from(format!("Day_{day:02}/input.txt")),
        PathBuf::from(format!("Day_{day:02}/input_{day:02}.txt")),
        PathBuf::from(format!("Day_{day:02}/input_{day}.txt")),
    ]
}

/// Split input into trimmed lines (keeps empty lines if present).
pub fn lines(input: &str) -> impl Iterator<Item = &str> {
    input.split('\n').map(|s| s.trim_end_matches('\r'))
}

/// Parse a whitespace-separated grid of integers into Vec<Vec<i64>>.
pub fn parse_int_grid(input: &str) -> Result<Vec<Vec<i64>>> {
    input
        .lines()
        .map(|line| {
            line.split_whitespace()
                .map(|tok| tok.parse::<i64>().map_err(|e| anyhow!(e)))
                .collect::<Result<Vec<_>>>()
        })
        .collect()
}

/// Helper to time a closure and return (result, elapsed_ms).
pub fn time<R, F: FnOnce() -> R>(f: F) -> (R, u128) {
    let start = std::time::Instant::now();
    let res = f();
    let elapsed = start.elapsed().as_millis();
    (res, elapsed)
}

/// Attempt to load session id from env var or SessionID.txt (day folder first, then repo root).
pub fn load_session(day: Option<u8>) -> Result<String> {
    if let Ok(env) = std::env::var("AOC_SESSION_ID") {
        let trimmed = env.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    let mut candidates = Vec::new();
    if let Some(d) = day {
        candidates.push(PathBuf::from(format!("Day_{d:02}/SessionID.txt")));
    }
    candidates.push(PathBuf::from("SessionID.txt"));

    for path in candidates {
        if let Ok(contents) = fs::read_to_string(&path) {
            let trimmed = contents.trim().to_string();
            if !trimmed.is_empty() {
                return Ok(trimmed);
            }
        }
    }

    Err(anyhow!(
        "Missing session cookie. Set AOC_SESSION_ID or place SessionID.txt in the day folder or repo root."
    ))
}

/// Load user agent string (env `AOC_USER_AGENT` or fallback).
pub fn load_user_agent() -> String {
    std::env::var("AOC_USER_AGENT")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| USER_AGENT_FALLBACK.to_string())
}

fn http_client(user_agent: &str) -> Result<Client> {
    Client::builder()
        .user_agent(user_agent)
        .build()
        .context("Building HTTP client")
}

/// Submission outcome variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmissionVerdict {
    Correct,
    TooLow,
    TooHigh,
    Wrong,
    TooSoon,
    AlreadySolved,
    Unknown(String),
}

impl std::fmt::Display for SubmissionVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubmissionVerdict::Correct => write!(f, "OK"),
            SubmissionVerdict::TooLow => write!(f, "WRONG (too low)"),
            SubmissionVerdict::TooHigh => write!(f, "WRONG (too high)"),
            SubmissionVerdict::Wrong => write!(f, "WRONG"),
            SubmissionVerdict::TooSoon => write!(f, "TOO MANY REQUESTS"),
            SubmissionVerdict::AlreadySolved => write!(f, "ALREADY SOLVED"),
            SubmissionVerdict::Unknown(s) => write!(f, "UNKNOWN ({s})"),
        }
    }
}

/// Submit an answer to AoC and classify the response.
pub fn submit_answer(
    day: u8,
    level: u8,
    answer: impl ToString,
    year: i32,
) -> Result<SubmissionVerdict> {
    let session = load_session(Some(day))?;
    let user_agent = load_user_agent();
    let client = http_client(&user_agent)?;

    let url = format!("https://adventofcode.com/{year}/day/{day}/answer");
    let resp = client
        .post(url)
        .header("Cookie", format!("session={session}"))
        .form(&[("level", level.to_string()), ("answer", answer.to_string())])
        .send()
        .context("Failed to submit answer")?;

    if !resp.status().is_success() {
        return Err(anyhow!("HTTP {} when submitting answer", resp.status()));
    }

    let text = resp.text().context("Reading submission response")?;
    let verdict = classify_submission(&text);
    Ok(verdict)
}

fn classify_submission(text: &str) -> SubmissionVerdict {
    if text.contains("That's the right answer!") {
        SubmissionVerdict::Correct
    } else if text.contains("You gave an answer too recently") {
        SubmissionVerdict::TooSoon
    } else if text.contains("You don't seem to be solving the right level.") {
        SubmissionVerdict::AlreadySolved
    } else if text.contains("not the right answer") {
        if text.contains("too low") {
            SubmissionVerdict::TooLow
        } else if text.contains("too high") {
            SubmissionVerdict::TooHigh
        } else {
            SubmissionVerdict::Wrong
        }
    } else {
        let snippet: String = text.chars().take(120).collect();
        SubmissionVerdict::Unknown(snippet)
    }
}

/// Detect part: returns 2 if `instructions-two.md` exists for the day, else 1.
pub fn detect_part(day: u8) -> u8 {
    let path = PathBuf::from(format!("Day_{day:02}/instructions-two.md"));
    if path.exists() {
        2
    } else {
        1
    }
}

/// Load example input if present.
pub fn load_example(day: u8) -> Result<String> {
    let candidates = vec![
        PathBuf::from(format!("Day_{day:02}/Example_{day:02}.txt")),
        PathBuf::from(format!("Day_{day:02}/example.txt")),
    ];
    for path in candidates {
        if let Ok(contents) = fs::read_to_string(&path) {
            return Ok(contents);
        }
    }
    Err(anyhow!("No example input found for day {day}"))
}

/// Simple prompt helper used before submissions.
pub fn confirm_prompt() -> Result<()> {
    print!("Press Enter to submit or Ctrl+C to abort... ");
    io::stdout().flush().ok();
    let mut buf = String::new();
    io::stdin()
        .read_line(&mut buf)
        .context("Reading confirmation input")?;
    Ok(())
}
