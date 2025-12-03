use std::collections::{HashMap, VecDeque};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;

pub const DEFAULT_YEAR: i32 = 2025;
const USER_AGENT_FALLBACK: &str =
    "github.com/your-handle/AdventOfCode_2025 (please set AOC_USER_AGENT with contact info)";

//##################################################################################################
// Input Fetching & Caching
//##################################################################################################

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
    for path in input_paths(day) {
        if let Ok(contents) = fs::read_to_string(&path) {
            return Some(contents);
        }
    }
    None
}

fn cache_input(day: u8, contents: &str) -> Result<()> {
    let path = canonical_input_path(day);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, contents)
        .with_context(|| format!("Failed to write input cache: {}", path.display()))?;
    Ok(())
}

fn canonical_input_path(day: u8) -> PathBuf {
    PathBuf::from(format!("Day_{day:02}/input_{day:02}.txt"))
}

fn input_paths(day: u8) -> Vec<PathBuf> {
    let mut paths = vec![canonical_input_path(day)];
    paths.push(PathBuf::from(format!("Day_{day:02}/input.txt")));
    paths.push(PathBuf::from(format!("Day_{day:02}/input_{day}.txt")));
    paths
}

//##################################################################################################
// Parsing Helpers
//##################################################################################################

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

//##################################################################################################
// Timing Helpers
//##################################################################################################

/// Helper to time a closure and return (result, elapsed_ms).
pub fn time<R, F: FnOnce() -> R>(f: F) -> (R, u128) {
    let start = std::time::Instant::now();
    let res = f();
    let elapsed = start.elapsed().as_millis();
    (res, elapsed)
}

/// Time a fallible closure and propagate its error, returning `(result, elapsed_ms)`.
pub fn time_result<R, F: FnOnce() -> Result<R>>(f: F) -> Result<(R, u128)> {
    let start = std::time::Instant::now();
    let res = f()?;
    let elapsed = start.elapsed().as_millis();
    Ok((res, elapsed))
}

//##################################################################################################
// Numeric Extraction
//##################################################################################################

/// Extract all signed integers from arbitrary text (useful when numbers are embedded in prose).
pub fn ints(input: &str) -> Vec<i64> {
    input
        .split(|c: char| !(c.is_ascii_digit() || c == '-'))
        .filter(|tok| !tok.is_empty() && tok != &"-")
        .filter_map(|tok| tok.parse::<i64>().ok())
        .collect()
}

/// Extract all unsigned integers from arbitrary text.
pub fn uints(input: &str) -> Vec<u64> {
    input
        .split(|c: char| !c.is_ascii_digit())
        .filter(|tok| !tok.is_empty())
        .filter_map(|tok| tok.parse::<u64>().ok())
        .collect()
}

/// Parse a string into individual numeric digits, ignoring any non-digit characters.
pub fn digits(input: &str) -> Vec<u8> {
    input
        .chars()
        .filter_map(|c| c.to_digit(10).map(|d| d as u8))
        .collect()
}

//##################################################################################################
// Math Utilities
//##################################################################################################

/// Greatest common divisor (Euclidean algorithm).
pub fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a.abs()
}

/// Least common multiple; returns 0 if either operand is 0.
pub fn lcm(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 {
        0
    } else {
        (a / gcd(a, b)) * b
    }
}

//##################################################################################################
// Grid Primitives
//##################################################################################################

/// Grid point with integer coordinates (x increases right, y increases down).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}

impl Point {
    /// Construct a new point.
    pub const fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    /// Manhattan distance to another point.
    pub fn manhattan(self, other: Point) -> i64 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    /// 4-neighborhood (right, left, down, up).
    pub fn neighbors4(self) -> [Point; 4] {
        [
            Point::new(self.x + 1, self.y),
            Point::new(self.x - 1, self.y),
            Point::new(self.x, self.y + 1),
            Point::new(self.x, self.y - 1),
        ]
    }

    /// 8-neighborhood (including diagonals).
    pub fn neighbors8(self) -> [Point; 8] {
        [
            Point::new(self.x + 1, self.y),
            Point::new(self.x - 1, self.y),
            Point::new(self.x, self.y + 1),
            Point::new(self.x, self.y - 1),
            Point::new(self.x + 1, self.y + 1),
            Point::new(self.x + 1, self.y - 1),
            Point::new(self.x - 1, self.y + 1),
            Point::new(self.x - 1, self.y - 1),
        ]
    }
}

/// Cardinal directions for grid problems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dir4 {
    Up,
    Down,
    Left,
    Right,
}

impl Dir4 {
    pub const ALL: [Dir4; 4] = [Dir4::Up, Dir4::Down, Dir4::Left, Dir4::Right];

    /// Return the delta vector for this direction.
    pub fn delta(self) -> Point {
        match self {
            Dir4::Up => Point::new(0, -1),
            Dir4::Down => Point::new(0, 1),
            Dir4::Left => Point::new(-1, 0),
            Dir4::Right => Point::new(1, 0),
        }
    }
}

//##################################################################################################
// Grid & Graph Helpers
//##################################################################################################

/// Add two points component-wise.
pub fn add_point(a: Point, b: Point) -> Point {
    Point::new(a.x + b.x, a.y + b.y)
}

/// Check whether a point lies inside a `width x height` rectangle (origin at top-left, exclusive upper bounds).
pub fn in_bounds(pt: Point, width: i64, height: i64) -> bool {
    pt.x >= 0 && pt.x < width && pt.y >= 0 && pt.y < height
}

/// Count frequency of items in an iterator; returns a `HashMap` of value -> count.
pub fn counts<T: Eq + std::hash::Hash>(iter: impl IntoIterator<Item = T>) -> HashMap<T, usize> {
    let mut map = HashMap::new();
    for item in iter {
        *map.entry(item).or_insert(0) += 1;
    }
    map
}

/// Multi-source BFS over an unweighted graph; returns a distance map from all starts.
pub fn bfs_distances<T, I, F>(
    starts: impl IntoIterator<Item = T>,
    mut neighbors: F,
) -> HashMap<T, usize>
where
    T: Eq + std::hash::Hash + Copy,
    F: FnMut(T) -> I,
    I: IntoIterator<Item = T>,
{
    let mut dist = HashMap::new();
    let mut q = VecDeque::new();

    for s in starts {
        dist.insert(s, 0);
        q.push_back(s);
    }

    while let Some(cur) = q.pop_front() {
        let next_d = dist[&cur] + 1;
        for nxt in neighbors(cur) {
            if dist.contains_key(&nxt) {
                continue;
            }
            dist.insert(nxt, next_d);
            q.push_back(nxt);
        }
    }

    dist
}

/// Simple Dijkstra; neighbors yield `(node, cost)` and the function returns the distance map.
/// Meant for small/medium AoC graphs—no early-exit target to keep the API minimal.
pub fn dijkstra<T, I, F>(start: T, mut neighbors: F) -> HashMap<T, u64>
where
    T: Eq + std::hash::Hash + Copy + Ord,
    F: FnMut(T) -> I,
    I: IntoIterator<Item = (T, u64)>,
{
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    let mut dist: HashMap<T, u64> = HashMap::new();
    let mut heap = BinaryHeap::new();
    dist.insert(start, 0);
    heap.push((Reverse(0u64), start));

    while let Some((Reverse(d), node)) = heap.pop() {
        if d != dist[&node] {
            continue; // stale entry
        }
        for (nxt, w) in neighbors(node) {
            let nd = d + w;
            let entry = dist.entry(nxt).or_insert(u64::MAX);
            if nd < *entry {
                *entry = nd;
                heap.push((Reverse(nd), nxt));
            }
        }
    }

    dist
}

/// Transpose a rectangular matrix (allocates a new Vec<Vec<T>>); panics if rows are ragged.
pub fn transpose<T: Clone>(grid: &[Vec<T>]) -> Vec<Vec<T>> {
    if grid.is_empty() {
        return Vec::new();
    }
    let rows = grid.len();
    let cols = grid[0].len();
    let mut out = vec![vec![grid[0][0].clone(); rows]; cols];
    for r in 0..rows {
        for c in 0..cols {
            out[c][r] = grid[r][c].clone();
        }
    }
    out
}

//##################################################################################################
// Session & Networking
//##################################################################################################

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

//##################################################################################################
// Submission Helpers
//##################################################################################################

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

//##################################################################################################
// Day Metadata & Examples
//##################################################################################################

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

//##################################################################################################
// UX Helpers
//##################################################################################################

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
