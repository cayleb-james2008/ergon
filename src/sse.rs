//! True Server-Sent Events (SSE) live run event bus and active run cancellation.
//!
//! Each in-flight agent run publishes structured events (`run_started`,
//! `step_start`, `step_done`, `file_written`, `approval_parked`,
//! `run_cancelling`, `run_finished`) onto a bounded ring buffer guarded by a
//! Condvar. Multiple concurrent `GET /api/runs/stream` clients receive events
//! in real time with periodic heartbeats and zero thread leaks on disconnect.

use serde_json::{json, Value};
use std::collections::VecDeque;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

const MAX_BUFFERED_EVENTS: usize = 256;

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
        let mut guard = lock.lock().unwrap();
        let seq = guard.next_seq;
        guard.next_seq += 1;
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

    pub fn head_seq(&self) -> u64 {
        let (lock, _) = &*self.inner;
        let guard = lock.lock().unwrap();
        guard.next_seq.saturating_sub(1)
    }

    pub fn wait_since(&self, after_seq: u64, timeout: Duration) -> Vec<RunEvent> {
        let (lock, cvar) = &*self.inner;
        let mut guard = lock.lock().unwrap();
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
        let (next_guard, _) = cvar.wait_timeout(guard, timeout).unwrap();
        guard = next_guard;
        collect(&guard)
    }
}

/// Serve a long-lived `text/event-stream` response until the client disconnects
/// (or `max_events` is reached when testing).
pub fn stream_sse(
    stream: &mut TcpStream,
    bus: &RunEventBus,
    person_id: &str,
    thread_filter: Option<&str>,
    initial_status: &Value,
    once: bool,
) -> std::io::Result<()> {
    let _ = stream.set_nodelay(true);
    let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));

    let header = "HTTP/1.1 200 OK\r\n\
                  Content-Type: text/event-stream; charset=utf-8\r\n\
                  Cache-Control: no-cache, no-transform\r\n\
                  Connection: keep-alive\r\n\
                  X-Accel-Buffering: no\r\n\
                  X-Content-Type-Options: nosniff\r\n\
                  X-Frame-Options: DENY\r\n\
                  Referrer-Policy: no-referrer\r\n\r\n";
    stream.write_all(header.as_bytes())?;

    let mut cursor = bus.head_seq();
    let init_payload = format!("event: status\ndata: {initial_status}\n\n");
    stream.write_all(init_payload.as_bytes())?;
    stream.flush()?;

    if once {
        return Ok(());
    }

    loop {
        let batch = bus.wait_since(cursor, Duration::from_secs(10));
        if batch.is_empty() {
            // Heartbeat detects dead peers promptly so idle threads exit cleanly.
            if stream.write_all(b": heartbeat\n\n").is_err() || stream.flush().is_err() {
                return Ok(());
            }
            continue;
        }
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
            let frame = format!("id: {}\nevent: {}\ndata: {}\n\n", ev.seq, ev.event, ev.data);
            if stream.write_all(frame.as_bytes()).is_err() || stream.flush().is_err() {
                return Ok(());
            }
        }
    }
}
