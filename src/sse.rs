//! True Server-Sent Events (SSE) live run event bus and active run cancellation.
//!
//! Each in-flight agent run publishes structured events (`run_started`,
//! `step_start`, `token_delta`, `step_done`, `file_written`, `approval_parked`,
//! `run_cancelling`, `run_finished`) onto a bounded ring buffer guarded by a
//! Condvar. Multiple concurrent `GET /api/runs/stream` clients receive events
//! in real time with periodic heartbeats and zero thread leaks on disconnect.

use serde_json::{json, Value};
use std::collections::VecDeque;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

const MAX_BUFFERED_EVENTS: usize = 1024;

#[derive(Clone, Debug)]
pub struct RunEvent {
    pub seq: u64,
    pub event: String,
    pub person_id: String,
    pub run_id: String,
    pub thread_id: String,
    pub data: Value,
}

struct BusInner {
    next_seq: u64,
    events: VecDeque<RunEvent>,
}

#[derive(Clone)]
pub struct RunEventBus {
    inner: Arc<(Mutex<BusInner>, Condvar)>,
}

impl Default for RunEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl RunEventBus {
    pub fn new() -> Self {
        Self {
            inner: Arc::new((
                Mutex::new(BusInner {
                    next_seq: 1,
                    events: VecDeque::with_capacity(MAX_BUFFERED_EVENTS),
                }),
                Condvar::new(),
            )),
        }
    }

    pub fn publish(
        &self,
        event: &str,
        person_id: &str,
        run_id: &str,
        thread_id: &str,
        mut data: Value,
    ) -> u64 {
        let (lock, cvar) = &*self.inner;
        let mut guard = lock.lock().unwrap_or_else(|p| p.into_inner());
        let seq = guard.next_seq;
        guard.next_seq += 1;
        if !data.is_object() {
            data = json!({ "payload": data });
        }
        if let Some(obj) = data.as_object_mut() {
            obj.entry("seq".to_string()).or_insert_with(|| json!(seq));
            obj.entry("event".to_string()).or_insert_with(|| json!(event));
            obj.entry("run_id".to_string()).or_insert_with(|| json!(run_id));
            obj.entry("thread_id".to_string())
                .or_insert_with(|| json!(thread_id));
        }
        if guard.events.len() >= MAX_BUFFERED_EVENTS {
            guard.events.pop_front();
        }
        guard.events.push_back(RunEvent {
            seq,
            event: event.to_string(),
            person_id: person_id.to_string(),
            run_id: run_id.to_string(),
            thread_id: thread_id.to_string(),
            data,
        });
        cvar.notify_all();
        seq
    }

    pub fn publish_token_delta(
        &self,
        person_id: &str,
        run_id: &str,
        thread_id: &str,
        delta: &str,
        step_index: usize,
    ) -> u64 {
        self.publish(
            "token_delta",
            person_id,
            run_id,
            thread_id,
            json!({
                "delta": delta,
                "step_index": step_index,
                "ts_ms": crate::records::now_millis(),
            }),
        )
    }

    pub fn head_seq(&self) -> u64 {
        let (lock, _) = &*self.inner;
        let guard = lock.lock().unwrap_or_else(|p| p.into_inner());
        guard.next_seq.saturating_sub(1)
    }

    pub fn wake_all(&self) {
        let (lock, cvar) = &*self.inner;
        let _guard = lock.lock().unwrap_or_else(|p| p.into_inner());
        cvar.notify_all();
    }

    pub fn wait_since(&self, after_seq: u64, timeout: Duration) -> Vec<RunEvent> {
        let (lock, cvar) = &*self.inner;
        let mut guard = lock.lock().unwrap_or_else(|p| p.into_inner());
        let collect = |g: &BusInner| -> Vec<RunEvent> {
            g.events
                .iter()
                .filter(|e| e.seq > after_seq)
                .cloned()
                .collect()
        };
        let immediate = collect(&guard);
        if !immediate.is_empty() {
            return immediate;
        }
        let (next_guard, _) = cvar
            .wait_timeout(guard, timeout)
            .unwrap_or_else(|p| p.into_inner());
        guard = next_guard;
        collect(&guard)
    }

    /// Return all buffered events strictly after `after_seq` without waiting.
    pub fn events_since(&self, after_seq: u64) -> Vec<RunEvent> {
        let (lock, _) = &*self.inner;
        let guard = lock.lock().unwrap_or_else(|p| p.into_inner());
        guard
            .events
            .iter()
            .filter(|e| e.seq > after_seq)
            .cloned()
            .collect()
    }
}

/// Serve a long-lived `text/event-stream` response until the client disconnects
/// (or after the initial status frame when `once` is true).
///
/// Captures `bus.head_seq()` BEFORE sending HTTP response headers so a client
/// that begins work as soon as headers arrive can never outrun cursor setup.
pub fn stream_sse(
    stream: &mut TcpStream,
    bus: &RunEventBus,
    person_id: &str,
    thread_filter: Option<&str>,
    initial_status: &Value,
    once: bool,
) -> std::io::Result<()> {
    let cursor = bus.head_seq();
    stream_sse_from(
        stream,
        bus,
        person_id,
        thread_filter,
        initial_status,
        cursor,
        once,
    )
}

/// Serve a long-lived `text/event-stream` response starting from an explicit
/// event-bus cursor (`initial_cursor`).
///
/// Callers should capture `initial_cursor` (either from `Last-Event-ID`/`since`
/// or from `bus.head_seq()`) BEFORE snapshotting run state and BEFORE writing
/// any bytes to `stream`. Headers and the initial `event: status` frame are
/// flushed atomically in a single write so subscribers are guaranteed zero gap
/// between handshake and live delivery.
pub fn stream_sse_from(
    stream: &mut TcpStream,
    bus: &RunEventBus,
    person_id: &str,
    thread_filter: Option<&str>,
    initial_status: &Value,
    initial_cursor: u64,
    once: bool,
) -> std::io::Result<()> {
    let _ = stream.set_nodelay(true);
    let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));

    let mut cursor = initial_cursor;
    let status_id = bus.head_seq().max(cursor);
    let initial_response = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/event-stream; charset=utf-8\r\n\
         Cache-Control: no-cache, no-transform\r\n\
         Connection: keep-alive\r\n\
         X-Accel-Buffering: no\r\n\
         X-Content-Type-Options: nosniff\r\n\
         X-Frame-Options: DENY\r\n\
         Referrer-Policy: no-referrer\r\n\r\n\
         id: {status_id}\n\
         event: status\n\
         data: {initial_status}\n\n"
    );
    stream.write_all(initial_response.as_bytes())?;
    stream.flush()?;

    if once {
        let replay = bus.events_since(cursor);
        for ev in replay {
            if ev.person_id != person_id {
                continue;
            }
            if let Some(tf) = thread_filter {
                if !tf.is_empty() && ev.thread_id != tf {
                    continue;
                }
            }
            let frame = format!("id: {}\nevent: {}\ndata: {}\n\n", ev.seq, ev.event, ev.data);
            if stream.write_all(frame.as_bytes()).is_err() || stream.flush().is_err() {
                return Ok(());
            }
        }
        return Ok(());
    }

    loop {
        if crate::server::shutdown_requested() {
            return Ok(());
        }
        let batch = bus.wait_since(cursor, Duration::from_secs(5));
        if crate::server::shutdown_requested() {
            return Ok(());
        }
        if batch.is_empty() {
            // Heartbeat detects dead peers promptly so idle threads exit cleanly.
            if stream.write_all(b": heartbeat\n\n").is_err() || stream.flush().is_err() {
                return Ok(());
            }
            continue;
        }
        let mut out = String::new();
        for ev in batch {
            cursor = cursor.max(ev.seq);
            if ev.person_id != person_id {
                continue;
            }
            if let Some(tf) = thread_filter {
                if !tf.is_empty() && ev.thread_id != tf {
                    continue;
                }
            }
            out.push_str(&format!(
                "id: {}\nevent: {}\ndata: {}\n\n",
                ev.seq, ev.event, ev.data
            ));
        }
        if !out.is_empty() && (stream.write_all(out.as_bytes()).is_err() || stream.flush().is_err()) {
            return Ok(());
        }
    }
}
