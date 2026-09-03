//! `/health` and `/metrics`, on a socket of their own.
//!
//! Railway needs a healthcheck or it cannot tell a wedged process from a quiet
//! one, and the BACKLOG has said so since P08. It is a hand-rolled HTTP/1.1
//! responder rather than a framework because it serves two fixed paths and
//! adding an HTTP stack to the agent would widen the dependency surface of the
//! one binary that holds a key.
//!
//! `/health` is **not** "the process is alive" — a process that has stopped
//! ticking is alive and useless. It reports unhealthy when the last tick is
//! older than three intervals, so a wedged mark reader or a stuck loop gets
//! restarted instead of quietly serving 200s.

use std::sync::atomic::Ordering;
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::runtime::Metrics;

pub struct Health {
    pub metrics: Arc<Metrics>,
    /// Ticks are expected at least this often.
    pub tick_seconds: u64,
    /// Set once the agent has booted far enough to tick.
    pub started_unix: i64,
}

impl Health {
    /// Healthy while the agent is still starting up, then only while it is
    /// actually ticking.
    pub fn is_healthy(&self, now_unix: i64) -> (bool, String) {
        let last = self.metrics.last_tick_unix.load(Ordering::Relaxed);
        let grace = (self.tick_seconds * 3) as i64;
        if last == 0 {
            let waited = now_unix.saturating_sub(self.started_unix);
            return if waited <= grace {
                (true, format!("starting, {waited}s since boot"))
            } else {
                (false, format!("no tick {waited}s after boot"))
            };
        }
        let age = now_unix.saturating_sub(last);
        if age <= grace {
            (true, format!("last tick {age}s ago"))
        } else {
            (false, format!("last tick {age}s ago, over {grace}s"))
        }
    }
}

fn response(status: &str, content_type: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

/// Serve until the process ends. Errors on individual connections are logged
/// and dropped: a malformed request must not take the agent down.
pub async fn serve(
    listener: TcpListener,
    health: Arc<Health>,
    now: impl Fn() -> i64 + Send + Sync + 'static,
) {
    loop {
        let (mut socket, peer) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "health: accept failed");
                continue;
            }
        };
        let mut buf = [0u8; 1024];
        let n = match socket.read(&mut buf).await {
            Ok(n) => n,
            Err(e) => {
                tracing::debug!(%peer, error = %e, "health: read failed");
                continue;
            }
        };
        let request = String::from_utf8_lossy(&buf[..n]);
        let path = request
            .split_whitespace()
            .nth(1)
            .unwrap_or("/")
            .split('?')
            .next()
            .unwrap_or("/")
            .to_string();

        let body = match path.as_str() {
            "/metrics" => response(
                "200 OK",
                "text/plain; version=0.0.4",
                &health.metrics.render(),
            ),
            "/health" | "/" => {
                let (ok, detail) = health.is_healthy(now());
                let payload = format!(
                    "{{\"ok\":{ok},\"detail\":\"{detail}\",\"ticks\":{}}}\n",
                    health.metrics.ticks_total.load(Ordering::Relaxed)
                );
                response(
                    if ok {
                        "200 OK"
                    } else {
                        "503 Service Unavailable"
                    },
                    "application/json",
                    &payload,
                )
            }
            _ => response("404 Not Found", "text/plain", "not found\n"),
        };
        if let Err(e) = socket.write_all(body.as_bytes()).await {
            tracing::debug!(%peer, error = %e, "health: write failed");
        }
        let _ = socket.shutdown().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn health(tick_seconds: u64) -> Health {
        Health {
            metrics: Arc::new(Metrics::default()),
            tick_seconds,
            started_unix: 1_000,
        }
    }

    /// A process that is alive but not ticking is not healthy. This is the
    /// whole reason the endpoint exists.
    #[test]
    fn a_process_that_stopped_ticking_is_unhealthy() {
        let h = health(60);
        h.metrics.last_tick_unix.store(2_000, Ordering::Relaxed);
        assert!(h.is_healthy(2_060).0, "one interval is fine");
        assert!(h.is_healthy(2_180).0, "three intervals is the edge");
        assert!(!h.is_healthy(2_181).0, "past three intervals is wedged");
    }

    /// Boot takes a moment, and a restart loop caused by an impatient
    /// healthcheck is worse than a slow start.
    #[test]
    fn startup_has_a_grace_period_but_not_an_infinite_one() {
        let h = health(60);
        assert!(h.is_healthy(1_000).0, "just booted");
        assert!(h.is_healthy(1_180).0, "still inside the grace period");
        let (ok, detail) = h.is_healthy(1_181);
        assert!(!ok, "a process that never ticks must not read healthy");
        assert!(detail.contains("no tick"), "{detail}");
    }
}
