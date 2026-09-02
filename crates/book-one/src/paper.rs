//! Paper mode writer (P08, B12). One file per calendar day, in the schema of
//! docs/11 §7. The daily file is *rendered* from that day's append-only tick
//! log (`<dir>/ticks/YYYY-MM-DD.jsonl`), so a restart never loses a day and a
//! re-render is idempotent. A past day can only ever receive a `no run` marker,
//! never tick data. The schema has no APY / APR / annualised field, so one
//! cannot be added by accident. Lines the runner cannot yet compute honestly
//! (hedge error, marked return) render as `—`, and the renderer refuses to
//! produce a file at all if the tick log claims intents it has no PnL for.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDate, Utc};

use crate::tick::TickRecord;

/// Field names of the rendered file, in order. The schema test reads this so
/// nobody adds a rate field without the test noticing.
#[allow(dead_code)] // read by the schema test; kept next to the renderer on purpose
pub const SCHEMA_FIELDS: [&str; 11] = [
    "date",
    "mark_source",
    "started",
    "ticks",
    "regime counts",
    "proposed",
    "veto reasons",
    "hedge error",
    "daily loss halt",
    "marked return",
    "notes",
];

/// Late ticks that straddle midnight are accepted for this long after the day
/// ended; anything older is a backfill attempt and is refused.
const LATE_TICK_GRACE_SECS: i64 = 5 * 60;

#[derive(Debug, thiserror::Error)]
pub enum PaperError {
    #[error("refusing to backfill: tick for {day} recorded on {today}")]
    Backfill { day: NaiveDate, today: NaiveDate },
    #[error("refusing to render {day}: {intents} intents / {would_send} would-send rows but PnL fields are placeholders until P06/P07")]
    PlaceholderPnl {
        day: NaiveDate,
        intents: usize,
        would_send: usize,
    },
    #[error("io {0}")]
    Io(#[from] std::io::Error),
    #[error("json {0}")]
    Json(#[from] serde_json::Error),
}

/// A day's ticks plus how many log lines could not be parsed (a truncated
/// line from a crash must be counted, never fatal).
#[derive(Debug, Default)]
pub struct TickLog {
    pub ticks: Vec<TickRecord>,
    pub unparseable: usize,
}

pub struct PaperStore {
    dir: PathBuf,
    tick_seconds: u64,
}

impl PaperStore {
    pub fn new(dir: PathBuf, tick_seconds: u64) -> Self {
        Self { dir, tick_seconds }
    }

    pub fn ensure_dirs(&self) -> Result<(), PaperError> {
        fs::create_dir_all(self.dir.join("ticks"))?;
        Ok(())
    }

    pub fn day_file(&self, day: NaiveDate) -> PathBuf {
        self.dir.join(format!("{}.md", day.format("%Y-%m-%d")))
    }

    fn tick_log(&self, day: NaiveDate) -> PathBuf {
        self.dir
            .join("ticks")
            .join(format!("{}.jsonl", day.format("%Y-%m-%d")))
    }

    /// Append one tick to its day's log as a single write. Refuses anything
    /// that is not today (or a late tick from the last minutes of yesterday).
    pub fn record_tick(&self, rec: &TickRecord, now: DateTime<Utc>) -> Result<(), PaperError> {
        let today = now.date_naive();
        if rec.day != today {
            let day_end = rec
                .day
                .succ_opt()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|dt| dt.and_utc().timestamp())
                .unwrap_or(i64::MIN);
            let late_but_honest =
                rec.day < today && now.timestamp() - day_end <= LATE_TICK_GRACE_SECS;
            if !late_but_honest {
                return Err(PaperError::Backfill {
                    day: rec.day,
                    today,
                });
            }
        }
        self.ensure_dirs()?;
        let mut line = serde_json::to_string(rec)?;
        line.push('\n');
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.tick_log(rec.day))?;
        f.write_all(line.as_bytes())?;
        Ok(())
    }

    pub fn read_ticks(&self, day: NaiveDate) -> Result<TickLog, PaperError> {
        let path = self.tick_log(day);
        if !path.exists() {
            return Ok(TickLog::default());
        }
        let text = fs::read_to_string(path)?;
        let mut log = TickLog::default();
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            match serde_json::from_str::<TickRecord>(line) {
                Ok(t) => log.ticks.push(t),
                Err(_) => log.unparseable += 1,
            }
        }
        log.ticks.sort_by_key(|t| t.ts_unix);
        Ok(log)
    }

    /// Render the day file from its tick log. Idempotent.
    pub fn render_day(&self, day: NaiveDate, mark_source: &str) -> Result<PathBuf, PaperError> {
        let log = self.read_ticks(day)?;
        let body = render(day, mark_source, &log, self.tick_seconds)?;
        let path = self.day_file(day);
        write_atomic(&path, &body)?;
        Ok(path)
    }

    /// Whether this directory has ever recorded `start` (a day file or a tick
    /// log). A fresh volume must not invent `no run` markers for days it
    /// never observed.
    pub fn is_seeded(&self, start: NaiveDate) -> Result<bool, PaperError> {
        Ok(self.day_file(start).exists() || !self.read_ticks(start)?.ticks.is_empty())
    }

    /// For every day in `[start, before)`: a day with ticks but no file is
    /// rendered; a day with neither gets the `no run` marker. Never touches a
    /// day that already has a file. Does nothing on an unseeded directory.
    pub fn mark_missing_days(
        &self,
        start: NaiveDate,
        before: NaiveDate,
        mark_source: &str,
    ) -> Result<Vec<NaiveDate>, PaperError> {
        let mut written = Vec::new();
        if !self.is_seeded(start)? {
            return Ok(written);
        }
        let mut d = start;
        while d < before {
            if !self.day_file(d).exists() {
                if self.read_ticks(d)?.ticks.is_empty() {
                    let body = format!(
                        "date: {}\nno run — no ticks recorded (runner was not running)\n",
                        d.format("%Y-%m-%d")
                    );
                    write_atomic(&self.day_file(d), &body)?;
                } else {
                    self.render_day(d, mark_source)?;
                }
                written.push(d);
            }
            d = match d.succ_opt() {
                Some(n) => n,
                None => break,
            };
        }
        Ok(written)
    }
}

fn write_atomic(path: &Path, body: &str) -> Result<(), PaperError> {
    let tmp = path.with_extension("md.tmp");
    fs::write(&tmp, body)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn hhmm(ts: i64) -> String {
    DateTime::<Utc>::from_timestamp(ts, 0)
        .map(|t| t.format("%H:%MZ").to_string())
        .unwrap_or_else(|| "--:--Z".to_string())
}

fn span(a: i64, b: i64) -> String {
    format!("{}–{}", hhmm(a), hhmm(b))
}

/// The docs/11 §7 schema, exactly. No rate field exists here on purpose.
pub fn render(
    day: NaiveDate,
    mark_source: &str,
    log: &TickLog,
    tick_seconds: u64,
) -> Result<String, PaperError> {
    let ticks = &log.ticks;
    if ticks.is_empty() {
        return Ok(format!(
            "date: {}\nno run — no ticks recorded{}\n",
            day.format("%Y-%m-%d"),
            if log.unparseable > 0 {
                format!(" ({} unparseable tick rows)", log.unparseable)
            } else {
                String::new()
            }
        ));
    }
    let day_start = day
        .and_hms_opt(0, 0, 0)
        .map(|d| d.and_utc().timestamp())
        .unwrap_or(0);
    let first = ticks.first().map(|t| t.ts_unix).unwrap_or(0);
    let last = ticks.last().map(|t| t.ts_unix).unwrap_or(0);

    let mut regimes: BTreeMap<&str, usize> = BTreeMap::new();
    let mut reasons: BTreeMap<String, usize> = BTreeMap::new();
    let (mut proposed, mut skipped, mut would_send, mut vetoed) = (0usize, 0usize, 0usize, 0usize);
    let (mut stale, mut unreadable, mut rpc_errors) = (0usize, 0usize, 0usize);
    let mut ages: Vec<i64> = Vec::new();
    let mut restarts: Vec<i64> = Vec::new();
    let mut runner_gaps: Vec<(i64, i64)> = Vec::new();
    let mut feed_stalls: Vec<(i64, i64)> = Vec::new();
    let mut stall_run: Option<(i64, i64, i64, usize)> = None; // (publish_time, from, to, count)
    let mut prev_ts: Option<i64> = None;

    for t in ticks {
        *regimes.entry(t.regime.as_str()).or_default() += 1;
        if t.intent != "skip" {
            proposed += 1;
        }
        match t.verdict.as_str() {
            "skip" => skipped += 1,
            "allow" => would_send += 1,
            "veto" => {
                vetoed += 1;
                if let Some(r) = &t.reason {
                    *reasons.entry(r.clone()).or_default() += 1;
                    if r == "StaleOracle" {
                        stale += 1;
                    }
                }
            }
            _ => {}
        }
        match &t.error {
            Some(e) if e.starts_with("rpc:") => {
                rpc_errors += 1;
                unreadable += 1;
            }
            Some(_) => unreadable += 1,
            None => {}
        }
        if let Some(a) = t.mark_age_s {
            ages.push(a);
        }
        if t.tick_id.ends_with("-000001") && t.ts_unix != first {
            restarts.push(t.ts_unix);
        }
        if let Some(p) = prev_ts {
            if t.ts_unix - p > 2 * tick_seconds as i64 {
                runner_gaps.push((p, t.ts_unix));
            }
        }
        prev_ts = Some(t.ts_unix);
        // three or more consecutive ticks with the same publish_time = the feed stopped moving
        if let Some(pt) = t.mark_publish_time {
            stall_run = match stall_run {
                Some((p, from, _to, n)) if p == pt => Some((p, from, t.ts_unix, n + 1)),
                Some((_, from, to, n)) => {
                    if n >= 3 {
                        feed_stalls.push((from, to));
                    }
                    Some((pt, t.ts_unix, t.ts_unix, 1))
                }
                None => Some((pt, t.ts_unix, t.ts_unix, 1)),
            };
        }
    }
    if let Some((_, from, to, n)) = stall_run {
        if n >= 3 {
            feed_stalls.push((from, to));
        }
    }

    // Fail closed: until P06/P07 supply positions and PnL, a day with intents
    // has no honest hedge-error or marked-return line, so no file is rendered.
    if proposed > 0 || would_send > 0 {
        return Err(PaperError::PlaceholderPnl {
            day,
            intents: proposed,
            would_send,
        });
    }

    let regime_line = ["chop", "trend", "halt"]
        .iter()
        .map(|r| format!("{r} {}", regimes.get(r).copied().unwrap_or(0)))
        .collect::<Vec<_>>()
        .join(" / ");
    let veto_line = if reasons.is_empty() {
        "—".to_string()
    } else {
        reasons
            .iter()
            .map(|(r, n)| format!("{r} {n}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let (age_med, age_max) = if ages.is_empty() {
        ("—".to_string(), "—".to_string())
    } else {
        let mut s = ages.clone();
        s.sort_unstable();
        (
            format!("{}s", s[s.len() / 2]),
            format!("{}s", s[s.len() - 1]),
        )
    };

    let mut notes: Vec<String> = Vec::new();
    if first > day_start + 60 {
        notes.push(format!("started late: first tick {}", hhmm(first)));
    }
    if !restarts.is_empty() {
        notes.push(format!(
            "{} restart(s) at {}",
            restarts.len(),
            restarts
                .iter()
                .map(|t| hhmm(*t))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !runner_gaps.is_empty() {
        let shown: Vec<String> = runner_gaps
            .iter()
            .take(5)
            .map(|(a, b)| span(*a, *b))
            .collect();
        notes.push(format!(
            "runner gap{} {}{}",
            if runner_gaps.len() > 1 { "s" } else { "" },
            shown.join(", "),
            if runner_gaps.len() > 5 {
                format!(" (+{} more)", runner_gaps.len() - 5)
            } else {
                String::new()
            }
        ));
    }
    if !feed_stalls.is_empty() {
        let shown: Vec<String> = feed_stalls
            .iter()
            .take(5)
            .map(|(a, b)| span(*a, *b))
            .collect();
        notes.push(format!("mark feed stalled {}", shown.join(", ")));
    }
    notes.push(format!("mark age median {age_med} max {age_max}"));
    notes.push(format!(
        "mark stale on {stale} ticks; mark unreadable on {unreadable} ticks; {rpc_errors} rpc errors"
    ));
    if log.unparseable > 0 {
        notes.push(format!("{} unparseable tick rows skipped", log.unparseable));
    }
    notes.push("core is the P06 placeholder (always skip); no positions, so hedge error and marked return are not computed".to_string());

    Ok(format!(
        "date: {date}\n\
         mark_source: {mark_source}\n\
         started: {started}   ended: {ended}   ticks: {ticks}\n\
         regime counts: {regime_line}\n\
         proposed: {proposed}   skipped: {skipped}   would_send: {would_send}   vetoed: {vetoed}\n\
         veto reasons: {veto_line}\n\
         hedge error (mean |target-actual| delta, USD): —   max: —\n\
         daily loss halt: no (no positions)\n\
         marked return: —          <- marked, devnet-shaped, not a rate\n\
         notes: {notes}\n",
        date = day.format("%Y-%m-%d"),
        started = hhmm(first),
        ended = hhmm(last),
        ticks = ticks.len(),
        notes = notes.join("; "),
    ))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::tick::TickRecord;

    fn rec_full(
        ts: i64,
        n: u64,
        verdict: &'static str,
        reason: Option<&str>,
        publish: i64,
    ) -> TickRecord {
        let dt = DateTime::<Utc>::from_timestamp(ts, 0).unwrap();
        TickRecord {
            tick_id: format!("{}-{n:06}", dt.format("%Y%m%dT%H%M%SZ")),
            ts_unix: ts,
            day: dt.date_naive(),
            slot: 1,
            mark_source: "replay".into(),
            mark_price: Some(100.0),
            mark_publish_time: Some(publish),
            mark_age_s: Some(ts - publish),
            mark_age_slots: Some(60),
            regime: "chop".to_string(),
            intent: "skip".to_string(),
            verdict: verdict.to_string(),
            reason: reason.map(str::to_string),
            latency_ms: 5,
            error: None,
        }
    }
    fn rec(ts: i64, verdict: &'static str, reason: Option<&str>) -> TickRecord {
        rec_full(ts, 7, verdict, reason, ts - 10)
    }

    const DAY_A: i64 = 1_788_400_000; // 2026-09-03T01:46:40Z
    const DAY_B: i64 = DAY_A + 86_400;
    const TS: u64 = 60;

    fn store(tmp: &tempfile::TempDir) -> PaperStore {
        PaperStore::new(tmp.path().to_path_buf(), TS)
    }

    #[test]
    fn one_file_per_day() {
        let tmp = tempfile::tempdir().unwrap();
        let store = store(&tmp);
        let now_a = DateTime::<Utc>::from_timestamp(DAY_A + 120, 0).unwrap();
        store.record_tick(&rec(DAY_A, "skip", None), now_a).unwrap();
        store
            .record_tick(&rec(DAY_A + 60, "skip", None), now_a)
            .unwrap();
        store.render_day(now_a.date_naive(), "replay").unwrap();
        let now_b = DateTime::<Utc>::from_timestamp(DAY_B + 120, 0).unwrap();
        store
            .record_tick(&rec(DAY_B, "veto", Some("StaleOracle")), now_b)
            .unwrap();
        store.render_day(now_b.date_naive(), "replay").unwrap();
        let files: Vec<_> = fs::read_dir(tmp.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|x| x == "md"))
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(files.len(), 2, "{files:?}");
        let a = fs::read_to_string(store.day_file(now_a.date_naive())).unwrap();
        assert!(a.contains("ticks: 2"), "{a}");
        assert!(a.contains("skipped: 2"), "{a}");
        let b = fs::read_to_string(store.day_file(now_b.date_naive())).unwrap();
        assert!(b.contains("vetoed: 1"), "{b}");
        assert!(b.contains("veto reasons: StaleOracle 1"), "{b}");
        store.render_day(now_a.date_naive(), "replay").unwrap();
        assert_eq!(
            a,
            fs::read_to_string(store.day_file(now_a.date_naive())).unwrap()
        );
    }

    #[test]
    fn no_backfill() {
        let tmp = tempfile::tempdir().unwrap();
        let store = store(&tmp);
        let now = DateTime::<Utc>::from_timestamp(DAY_B + 3_600, 0).unwrap();
        let err = store
            .record_tick(&rec(DAY_A, "skip", None), now)
            .unwrap_err();
        assert!(matches!(err, PaperError::Backfill { .. }), "{err}");
        assert!(store
            .read_ticks(rec(DAY_A, "skip", None).day)
            .unwrap()
            .ticks
            .is_empty());
        let late = rec(DAY_B - 30, "skip", None);
        let just_after = DateTime::<Utc>::from_timestamp(DAY_B + 60, 0).unwrap();
        assert!(store.record_tick(&late, just_after).is_ok());
    }

    #[test]
    fn schema_has_no_apy_field() {
        let day = NaiveDate::from_ymd_opt(2026, 9, 2).unwrap();
        let log = TickLog {
            ticks: vec![rec(DAY_A, "skip", None)],
            unparseable: 0,
        };
        let out = render(day, "replay", &log, TS).unwrap().to_lowercase();
        for banned in [
            "apy",
            "apr ",
            "annual",
            "yield",
            "guaranteed",
            "%/yr",
            "0.00%",
        ] {
            assert!(
                !out.contains(banned),
                "rendered paper contains {banned:?}:\n{out}"
            );
        }
        for banned in ["apy", "apr", "annual", "yield", "return_rate"] {
            assert!(
                SCHEMA_FIELDS.iter().all(|f| !f.contains(banned)),
                "schema field contains {banned}"
            );
        }
        for f in SCHEMA_FIELDS {
            assert!(
                out.contains(f),
                "rendered paper is missing field {f:?}:\n{out}"
            );
        }
        assert!(out.contains("marked, devnet-shaped, not a rate"));
    }

    #[test]
    fn missing_days_get_a_no_run_marker_and_data_days_are_untouched() {
        let tmp = tempfile::tempdir().unwrap();
        let store = store(&tmp);
        let now_a = DateTime::<Utc>::from_timestamp(DAY_A + 120, 0).unwrap();
        store.record_tick(&rec(DAY_A, "skip", None), now_a).unwrap();
        store.render_day(now_a.date_naive(), "replay").unwrap();
        let day_a = now_a.date_naive();
        let day_b = day_a.succ_opt().unwrap();
        let before = day_b.succ_opt().unwrap();
        // seeded at day_a: day_b (no file, no ticks) gets a marker; day_a untouched
        let written = store.mark_missing_days(day_a, before, "replay").unwrap();
        assert_eq!(written, vec![day_b]);
        let marker = fs::read_to_string(store.day_file(day_b)).unwrap();
        assert!(
            marker.starts_with(&format!("date: {}\nno run — ", day_b.format("%Y-%m-%d"))),
            "{marker}"
        );
        let a = fs::read_to_string(store.day_file(day_a)).unwrap();
        assert!(a.contains("ticks: 1"), "data day was overwritten:\n{a}");
    }

    #[test]
    fn unseeded_directory_gets_no_markers() {
        let tmp = tempfile::tempdir().unwrap();
        let store = store(&tmp);
        let day_a = DateTime::<Utc>::from_timestamp(DAY_A, 0)
            .unwrap()
            .date_naive();
        let before = day_a.succ_opt().unwrap().succ_opt().unwrap();
        assert!(store
            .mark_missing_days(day_a, before, "replay")
            .unwrap()
            .is_empty());
        assert!(!store.day_file(day_a).exists());
    }

    #[test]
    fn a_day_with_ticks_but_no_file_is_rendered_not_marked() {
        let tmp = tempfile::tempdir().unwrap();
        let store = store(&tmp);
        let now_a = DateTime::<Utc>::from_timestamp(DAY_A + 120, 0).unwrap();
        store.record_tick(&rec(DAY_A, "skip", None), now_a).unwrap();
        let day_a = now_a.date_naive();
        let written = store
            .mark_missing_days(day_a, day_a.succ_opt().unwrap(), "replay")
            .unwrap();
        assert_eq!(written, vec![day_a]);
        let a = fs::read_to_string(store.day_file(day_a)).unwrap();
        assert!(a.contains("ticks: 1"), "{a}");
    }

    #[test]
    fn truncated_line_is_counted_not_fatal() {
        let tmp = tempfile::tempdir().unwrap();
        let store = store(&tmp);
        let now_a = DateTime::<Utc>::from_timestamp(DAY_A + 120, 0).unwrap();
        store.record_tick(&rec(DAY_A, "skip", None), now_a).unwrap();
        let path = tmp
            .path()
            .join("ticks")
            .join(format!("{}.jsonl", now_a.date_naive().format("%Y-%m-%d")));
        let mut f = fs::OpenOptions::new().append(true).open(&path).unwrap();
        f.write_all(b"{\"tick_id\":\"20260902T0300\n").unwrap();
        store
            .record_tick(&rec(DAY_A + 60, "skip", None), now_a)
            .unwrap();
        let log = store.read_ticks(now_a.date_naive()).unwrap();
        assert_eq!(log.ticks.len(), 2);
        assert_eq!(log.unparseable, 1);
        let out = render(now_a.date_naive(), "replay", &log, TS).unwrap();
        assert!(out.contains("1 unparseable tick rows skipped"), "{out}");
    }

    #[test]
    fn placeholder_pnl_refuses_to_render_a_day_with_intents() {
        let day = NaiveDate::from_ymd_opt(2026, 9, 2).unwrap();
        let mut t = rec(DAY_A, "allow", None);
        t.intent = "open".to_string();
        let log = TickLog {
            ticks: vec![t],
            unparseable: 0,
        };
        assert!(matches!(
            render(day, "replay", &log, TS),
            Err(PaperError::PlaceholderPnl { .. })
        ));
    }

    #[test]
    fn started_late_restarts_gaps_and_stalls_are_noted_from_the_log() {
        let start = DAY_A; // 01:46:40Z, well after 00:00Z
        let day = DateTime::<Utc>::from_timestamp(start, 0)
            .unwrap()
            .date_naive();
        let mut ticks = vec![
            rec_full(start, 1, "skip", None, start - 10),
            rec_full(start + 60, 2, "skip", None, start - 10),
            rec_full(start + 120, 3, "skip", None, start - 10), // 3rd tick with the same publish_time
            rec_full(start + 600, 1, "skip", None, start + 590), // restart after a 480 s gap
            rec_full(start + 660, 2, "skip", None, start + 650),
        ];
        ticks.sort_by_key(|t| t.ts_unix);
        let log = TickLog {
            ticks,
            unparseable: 0,
        };
        let out = render(day, "replay", &log, TS).unwrap();
        let late = format!("started late: first tick {}", hhmm(start));
        let restart = format!("1 restart(s) at {}", hhmm(start + 600));
        let gap = format!("runner gap {}", span(start + 120, start + 600));
        let stall = format!("mark feed stalled {}", span(start, start + 120));
        for needle in [&late, &restart, &gap, &stall] {
            assert!(
                out.contains(needle.as_str()),
                "missing {needle:?} in:\n{out}"
            );
        }
        assert!(out.contains("ticks: 5"), "{out}");
    }
}
