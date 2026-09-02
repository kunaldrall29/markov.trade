//! Paper mode writer (P08, B12). One file per calendar day, in the schema of
//! docs/11 §7. The daily file is *rendered* from that day's append-only tick
//! log (`<dir>/ticks/YYYY-MM-DD.jsonl`), so a restart never loses a day and a
//! re-render is idempotent. A past day can only ever receive a `no run` marker,
//! never tick data. The schema has no APY / APR / annualised field, so one
//! cannot be added by accident.

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
    #[error("io {0}")]
    Io(#[from] std::io::Error),
    #[error("json {0}")]
    Json(#[from] serde_json::Error),
}

pub struct PaperStore {
    dir: PathBuf,
}

impl PaperStore {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
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

    /// Append one tick to its day's log. Refuses anything that is not today
    /// (or a late tick from the last few minutes of yesterday).
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
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.tick_log(rec.day))?;
        serde_json::to_writer(&mut f, rec)?;
        f.write_all(b"\n")?;
        Ok(())
    }

    pub fn read_ticks(&self, day: NaiveDate) -> Result<Vec<TickRecord>, PaperError> {
        let path = self.tick_log(day);
        if !path.exists() {
            return Ok(Vec::new());
        }
        let text = fs::read_to_string(path)?;
        let mut out = Vec::new();
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            out.push(serde_json::from_str(line)?);
        }
        Ok(out)
    }

    /// Render the day file from its tick log. Idempotent.
    pub fn render_day(
        &self,
        day: NaiveDate,
        mark_source: &str,
        started_at: Option<DateTime<Utc>>,
    ) -> Result<PathBuf, PaperError> {
        let ticks = self.read_ticks(day)?;
        let body = render(day, mark_source, &ticks, started_at);
        let path = self.day_file(day);
        write_atomic(&path, &body)?;
        Ok(path)
    }

    /// For every day in `[start, before)` with neither a day file nor ticks,
    /// write the `no run` marker. Never touches a day that has data.
    pub fn mark_missing_days(
        &self,
        start: NaiveDate,
        before: NaiveDate,
    ) -> Result<Vec<NaiveDate>, PaperError> {
        let mut written = Vec::new();
        let mut d = start;
        while d < before {
            if !self.day_file(d).exists() && self.read_ticks(d)?.is_empty() {
                let body = format!(
                    "date: {}\nno run — no ticks recorded (runner was not running)\n",
                    d.format("%Y-%m-%d")
                );
                write_atomic(&self.day_file(d), &body)?;
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

/// The docs/11 §7 schema, exactly. No rate field exists here on purpose.
pub fn render(
    day: NaiveDate,
    mark_source: &str,
    ticks: &[TickRecord],
    started_at: Option<DateTime<Utc>>,
) -> String {
    if ticks.is_empty() {
        return format!(
            "date: {}\nno run — no ticks recorded\n",
            day.format("%Y-%m-%d")
        );
    }
    let first = ticks.iter().map(|t| t.ts_unix).min().unwrap_or(0);
    let last = ticks.iter().map(|t| t.ts_unix).max().unwrap_or(0);
    let mut regimes: BTreeMap<&str, usize> = BTreeMap::new();
    let mut reasons: BTreeMap<String, usize> = BTreeMap::new();
    let (mut proposed, mut skipped, mut would_send, mut vetoed) = (0usize, 0usize, 0usize, 0usize);
    let (mut stale, mut unreadable, mut rpc_errors) = (0usize, 0usize, 0usize);
    let mut ages: Vec<i64> = Vec::new();
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
    if let Some(st) = started_at {
        if st.date_naive() == day
            && st.timestamp()
                > day
                    .and_hms_opt(0, 0, 0)
                    .map(|d| d.and_utc().timestamp())
                    .unwrap_or(0)
                    + 60
        {
            notes.push(format!(
                "started late: runner started {} (first tick {})",
                st.format("%Y-%m-%d %H:%MZ"),
                hhmm(first)
            ));
        }
    }
    notes.push(format!("mark age median {age_med} max {age_max}"));
    notes.push(format!("mark stale on {stale} ticks; mark unreadable on {unreadable} ticks; {rpc_errors} rpc errors"));
    notes.push("core is the P06 placeholder (always skip); positions 0, so hedge error and marked return are structurally zero".to_string());
    format!(
        "date: {date}\n\
         mark_source: {mark_source}\n\
         started: {started}   ended: {ended}   ticks: {ticks}\n\
         regime counts: {regime_line}\n\
         proposed: {proposed}   skipped: {skipped}   would_send: {would_send}   vetoed: {vetoed}\n\
         veto reasons: {veto_line}\n\
         hedge error (mean |target-actual| delta, USD): —   max: —\n\
         daily loss halt: no\n\
         marked return: 0.00%          <- marked, devnet-shaped, not a rate\n\
         notes: {notes}\n",
        date = day.format("%Y-%m-%d"),
        started = hhmm(first),
        ended = hhmm(last),
        ticks = ticks.len(),
        notes = notes.join("; "),
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::tick::TickRecord;

    fn rec(ts: i64, verdict: &'static str, reason: Option<&str>) -> TickRecord {
        let dt = DateTime::<Utc>::from_timestamp(ts, 0).unwrap();
        TickRecord {
            tick_id: format!("t{ts}"),
            ts_unix: ts,
            day: dt.date_naive(),
            slot: 1,
            mark_source: "replay".into(),
            mark_price: Some(100.0),
            mark_publish_time: Some(ts - 10),
            mark_age_s: Some(10),
            mark_age_slots: Some(60),
            regime: "chop".to_string(),
            intent: "skip".to_string(),
            verdict: verdict.to_string(),
            reason: reason.map(str::to_string),
            latency_ms: 5,
            error: None,
        }
    }

    const DAY_A: i64 = 1_788_400_000; // 2026-09-02T02:26:40Z
    const DAY_B: i64 = DAY_A + 86_400;

    #[test]
    fn one_file_per_day() {
        let tmp = tempfile::tempdir().unwrap();
        let store = PaperStore::new(tmp.path().to_path_buf());
        let now_a = DateTime::<Utc>::from_timestamp(DAY_A + 120, 0).unwrap();
        store.record_tick(&rec(DAY_A, "skip", None), now_a).unwrap();
        store
            .record_tick(&rec(DAY_A + 60, "skip", None), now_a)
            .unwrap();
        store
            .render_day(now_a.date_naive(), "replay", None)
            .unwrap();
        let now_b = DateTime::<Utc>::from_timestamp(DAY_B + 120, 0).unwrap();
        store
            .record_tick(&rec(DAY_B, "veto", Some("StaleOracle")), now_b)
            .unwrap();
        store
            .render_day(now_b.date_naive(), "replay", None)
            .unwrap();
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
        // re-rendering is idempotent
        store
            .render_day(now_a.date_naive(), "replay", None)
            .unwrap();
        assert_eq!(
            a,
            fs::read_to_string(store.day_file(now_a.date_naive())).unwrap()
        );
    }

    #[test]
    fn no_backfill() {
        let tmp = tempfile::tempdir().unwrap();
        let store = PaperStore::new(tmp.path().to_path_buf());
        let now = DateTime::<Utc>::from_timestamp(DAY_B + 3_600, 0).unwrap();
        let err = store
            .record_tick(&rec(DAY_A, "skip", None), now)
            .unwrap_err();
        assert!(matches!(err, PaperError::Backfill { .. }), "{err}");
        assert!(store
            .read_ticks(rec(DAY_A, "skip", None).day)
            .unwrap()
            .is_empty());
        // a tick from the last minutes of yesterday, processed just after midnight, is honest
        let late = rec(DAY_B - 30, "skip", None);
        let just_after = DateTime::<Utc>::from_timestamp(DAY_B + 60, 0).unwrap();
        assert!(store.record_tick(&late, just_after).is_ok());
    }

    #[test]
    fn schema_has_no_apy_field() {
        let day = NaiveDate::from_ymd_opt(2026, 9, 2).unwrap();
        let out = render(day, "replay", &[rec(DAY_A, "skip", None)], None).to_lowercase();
        for banned in ["apy", "apr ", "annual", "yield", "guaranteed", "%/yr"] {
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
        let store = PaperStore::new(tmp.path().to_path_buf());
        let now_a = DateTime::<Utc>::from_timestamp(DAY_A + 120, 0).unwrap();
        store.record_tick(&rec(DAY_A, "skip", None), now_a).unwrap();
        store
            .render_day(now_a.date_naive(), "replay", None)
            .unwrap();
        let day_a = now_a.date_naive();
        let start = day_a.pred_opt().unwrap();
        let before = day_a.succ_opt().unwrap().succ_opt().unwrap(); // day A + 2
        let written = store.mark_missing_days(start, before).unwrap();
        assert_eq!(written, vec![start, day_a.succ_opt().unwrap()]);
        let marker = fs::read_to_string(store.day_file(start)).unwrap();
        assert!(
            marker.starts_with(&format!("date: {}\nno run — ", start.format("%Y-%m-%d"))),
            "{marker}"
        );
        let a = fs::read_to_string(store.day_file(day_a)).unwrap();
        assert!(a.contains("ticks: 1"), "data day was overwritten:\n{a}");
    }
}
