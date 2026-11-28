use super::*;

pub(super) fn is_history_security_error(err: &str) -> bool {
    let low = err.to_ascii_lowercase();
    low.contains("escapes")
        || low.contains("invalid")
        || low.contains("absolute")
        || low.contains("symlink")
        || low.contains("hidden")
        || low.contains("control")
        || low.contains("traversal")
        || low.contains("nul byte")
        || low.contains("null byte")
        || low.contains("escapes the workspace")
        || low.contains("inside the workspace")
}

pub(super) fn respond_checked_write_err(
    stream: &mut ConnStream,
    err: file_history::CheckedWriteErr,
) -> std::io::Result<()> {
    match err {
        file_history::CheckedWriteErr::BadRequest(msg) => respond_err(stream, 400, &msg),
        file_history::CheckedWriteErr::Forbidden(msg) => respond_err(stream, 403, &msg),
        file_history::CheckedWriteErr::NotFound(msg) => respond_err(stream, 404, &msg),
        file_history::CheckedWriteErr::Conflict {
            path,
            expected,
            actual,
            message,
        } => respond_json(
            stream,
            409,
            &json!({
                "error": message,
                "path": path,
                "expected_sha256": expected,
                "actual_sha256": actual,
                "current_sha256": actual,
            }),
        ),
        file_history::CheckedWriteErr::Internal(msg) => respond_err(stream, 500, &msg),
    }
}

pub(super) const MAX_PATCH_OPS: usize = 64;

pub(super) fn apply_patch_ops(orig: &str, ops: &[Value]) -> Result<(String, usize), String> {
    if ops.len() > MAX_PATCH_OPS {
        return Err(format!("too many patch ops (max {MAX_PATCH_OPS})"));
    }
    let mut working = orig.to_string();
    let mut applied = 0usize;
    for (idx, op_val) in ops.iter().enumerate() {
        let op = op_val["op"]
            .as_str()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        match op.as_str() {
            "replace" => {
                let find = op_val["find"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| format!("op[{idx}] 'replace' requires a non-empty 'find' string"))?;
                let replace = op_val["replace"]
                    .as_str()
                    .ok_or_else(|| format!("op[{idx}] 'replace' requires a 'replace' string"))?;
                let replace_all = op_val["replace_all"].as_bool().unwrap_or(false);
                let count = working.match_indices(find).count();
                if count == 0 {
                    return Err(format!("op[{idx}] 'replace': target text not found"));
                }
                if count > 1 && !replace_all {
                    return Err(format!(
                        "op[{idx}] 'replace': target matched {count} times (ambiguous); pass replace_all: true"
                    ));
                }
                working = if replace_all {
                    working.replace(find, replace)
                } else {
                    working.replacen(find, replace, 1)
                };
                applied += count;
            }
            "replace_lines" => {
                let start = op_val["start_line"]
                    .as_u64()
                    .ok_or_else(|| format!("op[{idx}] 'replace_lines' requires 1-based 'start_line'"))?
                    as usize;
                let end = op_val["end_line"].as_u64().unwrap_or(start as u64) as usize;
                let replace = op_val["replace"]
                    .as_str()
                    .ok_or_else(|| format!("op[{idx}] 'replace_lines' requires a 'replace' string"))?;
                if start == 0 || end < start {
                    return Err(format!("op[{idx}] 'replace_lines': invalid line range {start}..={end}"));
                }
                let had_nl = working.ends_with('\n');
                let body = working
                    .strip_suffix("\r\n")
                    .or_else(|| working.strip_suffix('\n'))
                    .unwrap_or(&working);
                let mut lines: Vec<String> = if working.is_empty() {
                    Vec::new()
                } else {
                    body.split('\n')
                        .map(|s| s.strip_suffix('\r').unwrap_or(s).to_string())
                        .collect()
                };
                if end > lines.len() {
                    return Err(format!(
                        "op[{idx}] 'replace_lines': end_line {end} exceeds line count {}",
                        lines.len()
                    ));
                }
                let rep_lines: Vec<String> = if replace.is_empty() {
                    Vec::new()
                } else {
                    let trimmed = replace
                        .strip_suffix("\r\n")
                        .or_else(|| replace.strip_suffix('\n'))
                        .unwrap_or(replace);
                    trimmed
                        .split('\n')
                        .map(|s| s.strip_suffix('\r').unwrap_or(s).to_string())
                        .collect()
                };
                lines.splice((start - 1)..end, rep_lines);
                working = lines.join("\n");
                if had_nl && !working.is_empty() {
                    working.push('\n');
                }
                applied += 1;
            }
            other => {
                return Err(format!(
                    "op[{idx}] unknown patch op '{other}' (use 'replace' or 'replace_lines')"
                ));
            }
        }
        if working.len() as u64 > docs::MAX_DOC_BYTES {
            return Err("patched file exceeds maximum allowed size".to_string());
        }
    }
    Ok((working, applied))
}


pub(crate) struct Request {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) query: String,
    pub(crate) body: Vec<u8>,
    /// The caller's `ergon_session` cookie, if they presented one.
    pub(crate) session: Option<String>,
    /// The hosting layer's gate header, if presented.
    pub(crate) gate: Option<String>,
    /// Explicit API token (`X-Ergon-Token`) / admin token (`X-Ergon-Admin-Token`)
    /// / hosting-layer role (`X-Ergon-Role`).
    pub(crate) api_token: Option<String>,
    pub(crate) api_key_header: Option<String>,
    pub(crate) admin_token: Option<String>,
    pub(crate) actor_role: Option<String>,
    /// Browser-set headers the cross-site checks read (lowercased).
    pub(crate) content_type: String,
    pub(crate) origin: Option<String>,
    pub(crate) host: Option<String>,
    pub(crate) sec_fetch_site: Option<String>,
    /// Headers the tripwire reads (honeytoken use, decoy cookie, proxies).
    pub(crate) authorization: Option<String>,
    pub(crate) user_agent: Option<String>,
    pub(crate) cookie_raw: Option<String>,
    pub(crate) forwarded_for: Option<String>,
    /// Content negotiation & conditional GET headers.
    pub(crate) accept_encoding: Option<String>,
    pub(crate) if_none_match: Option<String>,
    /// SSE resume header (`Last-Event-ID`).
    pub(crate) last_event_id: Option<String>,
    /// Webhook authentication headers for `POST /api/hooks/:hook_id`.
    pub(crate) hook_signature: Option<String>,
    pub(crate) hook_token: Option<String>,
    pub(crate) hook_timestamp: Option<String>,
    pub(crate) hook_delivery: Option<String>,
    pub(crate) keep_alive: bool,
}

thread_local! {
    pub(super) static KEEP_ALIVE_CONN: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    pub(super) static TLS_CONN: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    pub(super) static JSON_SCRATCH: std::cell::RefCell<Vec<u8>> = std::cell::RefCell::new(Vec::with_capacity(2048));
}

/// Bodies are small JSON. A request claiming a larger one is dropped
/// before any allocation: an unbounded `vec![0u8; content_length]`
/// here once let a single header (Content-Length: 2^40, no body)
/// abort the whole server process.
pub(super) const MAX_BODY_BYTES: usize = 1_048_576;
/// File uploads only: a 25 MB file, base64-encoded inside JSON. Every
/// other route keeps the 1 MB cap.
pub(super) const MAX_UPLOAD_BODY_BYTES: usize = 36 * 1_048_576;
pub(super) const MAX_HEAD_BYTES: usize = 32_768;

pub(super) enum ConnStream {
    Plain(std::net::TcpStream),
    Tls(Box<rustls::StreamOwned<rustls::ServerConnection, std::net::TcpStream>>),
}

impl ConnStream {
    pub(super) fn tcp(&self) -> &std::net::TcpStream {
        match self {
            ConnStream::Plain(s) => s,
            ConnStream::Tls(t) => t.get_ref(),
        }
    }

    pub(super) fn shutdown_both(&self) {
        let _ = self.tcp().shutdown(std::net::Shutdown::Both);
    }
}

impl Read for ConnStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            ConnStream::Plain(s) => s.read(buf),
            ConnStream::Tls(t) => t.read(buf),
        }
    }
}

impl Write for ConnStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            ConnStream::Plain(s) => s.write(buf),
            ConnStream::Tls(t) => t.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            ConnStream::Plain(s) => s.flush(),
            ConnStream::Tls(t) => t.flush(),
        }
    }
}

impl sse::SseSocket for ConnStream {
    fn prepare_sse(&self) {
        let s = self.tcp();
        let _ = s.set_nodelay(true);
        let _ = s.set_write_timeout(Some(std::time::Duration::from_secs(10)));
        let _ = s.set_read_timeout(Some(std::time::Duration::from_secs(10)));
    }
}

pub(super) trait SockStream: Read + Write {
    fn set_rt(&self, dur: std::time::Duration) -> std::io::Result<()>;
    fn shutdown_write(&self);
}

impl SockStream for ConnStream {
    fn set_rt(&self, dur: std::time::Duration) -> std::io::Result<()> {
        self.tcp().set_read_timeout(Some(dur))
    }
    fn shutdown_write(&self) {
        let _ = self.tcp().shutdown(std::net::Shutdown::Write);
    }
}

impl SockStream for std::net::TcpStream {
    fn set_rt(&self, dur: std::time::Duration) -> std::io::Result<()> {
        self.set_read_timeout(Some(dur))
    }
    fn shutdown_write(&self) {
        let _ = self.shutdown(std::net::Shutdown::Write);
    }
}

impl SockStream for &mut std::net::TcpStream {
    fn set_rt(&self, dur: std::time::Duration) -> std::io::Result<()> {
        self.set_read_timeout(Some(dur))
    }
    fn shutdown_write(&self) {
        let _ = self.shutdown(std::net::Shutdown::Write);
    }
}

#[cfg(test)]
pub(crate) fn read_request(stream: &mut std::net::TcpStream) -> Option<Request> {
    let mut reader = BufReader::with_capacity(2048, stream);
    read_request_from(&mut reader, std::time::Duration::from_secs(30), false)
}

#[inline]
pub(super) fn read_request_from<S: SockStream>(
    reader: &mut BufReader<S>,
    initial_timeout: std::time::Duration,
    idle_keep_alive: bool,
) -> Option<Request> {
    let deadline = std::cell::Cell::new(std::time::Instant::now() + initial_timeout);
    let arm = |s: &S| -> Option<()> {
        let remain = deadline.get().checked_duration_since(std::time::Instant::now())?;
        s.set_rt(remain).ok()?;
        Some(())
    };
    let mut line_buf = Vec::with_capacity(256);
    let n_req = loop {
        if idle_keep_alive && line_buf.is_empty() && shutdown_requested() {
            return None;
        }
        let remain = deadline.get().checked_duration_since(std::time::Instant::now())?;
        let slice = if idle_keep_alive && line_buf.is_empty() {
            remain.min(std::time::Duration::from_millis(100))
        } else {
            remain
        };
        reader.get_ref().set_rt(slice).ok()?;
        let room = MAX_HEAD_BYTES.saturating_sub(line_buf.len());
        let before = line_buf.len();
        match (&mut *reader)
            .take(room as u64 + 1)
            .read_until(b'\n', &mut line_buf)
        {
            Ok(0) => return None,
            Ok(_) => {
                if line_buf.len() > MAX_HEAD_BYTES {
                    return refuse_head(reader);
                }
                if line_buf.ends_with(b"\n") {
                    break line_buf.len();
                }
                if line_buf.len() >= MAX_HEAD_BYTES {
                    return refuse_head(reader);
                }
                return None;
            }
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                if line_buf.len() > before {
                    deadline.set(std::time::Instant::now() + std::time::Duration::from_secs(30));
                    continue;
                }
                if idle_keep_alive && line_buf.is_empty() {
                    continue;
                }
                return None;
            }
            Err(_) => return None,
        }
    };
    // Once request-line bytes arrived, give the remaining head up to 30s total.
    deadline.set(std::time::Instant::now() + std::time::Duration::from_secs(30));
    let request_line = std::str::from_utf8(&line_buf)
        .map(|s| s.to_string())
        .unwrap_or_else(|_| String::from_utf8_lossy(&line_buf).into_owned());
    let mut parts = request_line.split_whitespace();
    let method = parts.next()?.to_string();
    let raw_path = parts.next()?.to_string();
    let proto = parts.next().unwrap_or("HTTP/1.1");
    let mut conn_close = false;
    let mut conn_keep_alive = false;
    let (path_part, query_part) = match raw_path.split_once('?') {
        Some((p, q)) => (p, q),
        None => (raw_path.as_str(), ""),
    };
    let path = path_part.to_string();
    let query = query_part.to_string();
    let mut content_length: Option<usize> = None;
    let mut bad_length = false;
    let mut bad_host = false;
    let mut chunked = false;
    let mut session: Option<String> = None;
    let mut gate: Option<String> = None;
    let mut api_token: Option<String> = None;
    let mut api_key_header: Option<String> = None;
    let mut admin_token: Option<String> = None;
    let mut actor_role: Option<String> = None;
    let mut content_type = String::new();
    let mut origin: Option<String> = None;
    let mut host: Option<String> = None;
    let mut sec_fetch_site: Option<String> = None;
    let mut authorization: Option<String> = None;
    let mut user_agent: Option<String> = None;
    let mut cookie_raw: Option<String> = None;
    let mut forwarded_for: Option<String> = None;
    let mut accept_encoding: Option<String> = None;
    let mut if_none_match: Option<String> = None;
    let mut last_event_id: Option<String> = None;
    let mut hook_signature: Option<String> = None;
    let mut hook_token: Option<String> = None;
    let mut hook_timestamp: Option<String> = None;
    let mut hook_delivery: Option<String> = None;
    let mut head_bytes = n_req;
    loop {
        arm(reader.get_ref())?;
        let room = MAX_HEAD_BYTES - head_bytes;
        let n = match head_line_into(reader, room, &mut line_buf) {
            Ok(Some(read)) => read,
            Ok(None) => return refuse_head(reader),
            Err(_) => return None,
        };
        if n == 0 || line_buf == b"\r\n" || line_buf == b"\n" {
            break;
        }
        head_bytes += n;
        let line_cow = String::from_utf8_lossy(&line_buf);
        let Some((raw_name, raw_val)) = line_cow.split_once(':') else {
            continue;
        };
        let name = raw_name.trim();
        let val = raw_val.trim();
        if name.eq_ignore_ascii_case("content-length") {
            match val.parse::<usize>() {
                Ok(len) => {
                    if let Some(prev) = content_length {
                        if prev != len {
                            bad_length = true;
                        }
                    } else {
                        content_length = Some(len);
                    }
                }
                Err(_) => bad_length = true,
            }
        } else if name.eq_ignore_ascii_case("transfer-encoding") {
            if val.to_ascii_lowercase().contains("chunked") {
                chunked = true;
            }
        } else if name.eq_ignore_ascii_case("content-type") {
            content_type = val.to_ascii_lowercase();
        } else if name.eq_ignore_ascii_case("origin") {
            origin = Some(val.to_ascii_lowercase());
        } else if name.eq_ignore_ascii_case("host") {
            if host.is_some() {
                bad_host = true;
            } else {
                host = Some(val.to_ascii_lowercase());
            }
        } else if name.eq_ignore_ascii_case("sec-fetch-site") {
            sec_fetch_site = Some(val.to_ascii_lowercase());
        } else if name.eq_ignore_ascii_case("accept-encoding") {
            accept_encoding = Some(val.to_string());
        } else if name.eq_ignore_ascii_case("if-none-match") {
            if_none_match = Some(val.to_string());
        } else if name.eq_ignore_ascii_case("last-event-id") && !val.is_empty() {
            last_event_id = Some(val.to_string());
        } else if name.eq_ignore_ascii_case("authorization") {
            authorization = Some(val.to_string());
        } else if name.eq_ignore_ascii_case("user-agent") {
            user_agent = Some(val.to_string());
        } else if name.eq_ignore_ascii_case("x-forwarded-for") {
            forwarded_for = Some(val.to_string());
        } else if name.eq_ignore_ascii_case("cookie") {
            cookie_raw = Some(val.to_string());
            for pair in val.split(';') {
                let pair = pair.trim();
                if let Some((k, id)) = pair.split_once('=') {
                    if k.trim().eq_ignore_ascii_case("ergon_session") {
                        let id = id.trim().to_string();
                        if !id.is_empty() {
                            session = Some(id);
                        }
                    }
                }
            }
        } else if name.eq_ignore_ascii_case("connection") {
            let low = val.to_ascii_lowercase();
            if low.contains("close") {
                conn_close = true;
            } else if low.contains("keep-alive") {
                conn_keep_alive = true;
            }
        } else if name.eq_ignore_ascii_case("x-ergon-gate") && !val.is_empty() {
            gate = Some(val.to_string());
        } else if name.eq_ignore_ascii_case("x-ergon-token") && !val.is_empty() {
            api_token = Some(val.to_string());
        } else if name.eq_ignore_ascii_case("x-ergon-api-key") && !val.is_empty() {
            api_key_header = Some(val.to_string());
        } else if name.eq_ignore_ascii_case("x-ergon-admin-token") && !val.is_empty() {
            admin_token = Some(val.to_string());
        } else if name.eq_ignore_ascii_case("x-ergon-role") && !val.is_empty() {
            actor_role = Some(val.to_string());
        } else if name.eq_ignore_ascii_case("x-ergon-signature") && !val.is_empty() {
            hook_signature = Some(val.to_string());
        } else if name.eq_ignore_ascii_case("x-ergon-hook-token") && !val.is_empty() {
            hook_token = Some(val.to_string());
        } else if name.eq_ignore_ascii_case("x-ergon-timestamp") && !val.is_empty() {
            hook_timestamp = Some(val.to_string());
        } else if name.eq_ignore_ascii_case("x-ergon-delivery") && !val.is_empty() {
            hook_delivery = Some(val.to_string());
        }
    }
    if bad_length || bad_host || chunked {
        let msg = if chunked {
            r#"{"error":"chunked transfer-encoding is not supported"}"#
        } else if bad_host {
            r#"{"error":"multiple Host headers are not permitted"}"#
        } else {
            r#"{"error":"invalid Content-Length header"}"#
        };
        let response = format!(
            "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{msg}",
            msg.len()
        );
        let s = reader.get_mut();
        let _ = s.write_all(response.as_bytes());
        let _ = s.flush();
        s.shutdown_write();
        return None;
    }
    let content_length = content_length.unwrap_or(0);
    let body_limit = if method == "POST" && path == "/api/files/upload" {
        MAX_UPLOAD_BODY_BYTES
    } else if matches!(method.as_str(), "PUT" | "PATCH") && path == "/api/files/content" {
        4 * 1_048_576
    } else if method == "POST" && path == "/api/verify/evidence" {
        16 * 1024 * 1024
    } else if method == "POST" && path == "/api/playbooks/import" {
        routines::MAX_PLAYBOOK_PACK_BYTES
    } else {
        MAX_BODY_BYTES
    };
    if content_length > body_limit {
        // Drain only already-arrived / fast in-flight body bytes using a short 50ms timeout
        // (and 250ms total cap) so real uploads don't get RST while forged huge Content-Length
        // headers (e.g. 99999999999999) with an open write-half return 413 promptly.
        let mut to_drain = content_length.min(4 * 1024 * 1024);
        let drain_end = std::time::Instant::now() + std::time::Duration::from_millis(250);
        let _ = reader
            .get_ref()
            .set_rt(std::time::Duration::from_millis(50));
        let mut sink = [0u8; 8192];
        while to_drain > 0 && std::time::Instant::now() < drain_end {
            let want = to_drain.min(sink.len());
            match reader.read(&mut sink[..want]) {
                Ok(0) => break,
                Ok(n) => to_drain -= n,
                Err(_) => break,
            }
        }
        let body = format!(
            "{{\"error\": \"request body too large: the limit is {body_limit} bytes\"}}"
        );
        let response = format!(
            "HTTP/1.1 413 Content Too Large\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let s = reader.get_mut();
        let _ = s.write_all(response.as_bytes());
        let _ = s.flush();
        s.shutdown_write();
        return None;
    }
    if body_limit == MAX_UPLOAD_BODY_BYTES && content_length > MAX_BODY_BYTES {
        deadline.set(std::time::Instant::now() + std::time::Duration::from_secs(180));
    }
    arm(reader.get_ref())?;
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body).ok()?;
    }
    Some(Request {
        method,
        path,
        query,
        body,
        session,
        gate,
        api_token,
        api_key_header,
        admin_token,
        actor_role,
        content_type,
        origin,
        host,
        sec_fetch_site,
        authorization,
        user_agent,
        cookie_raw,
        forwarded_for,
        accept_encoding,
        if_none_match,
        last_event_id,
        hook_signature,
        hook_token,
        hook_timestamp,
        hook_delivery,
        keep_alive: if proto == "HTTP/1.0" {
            conn_keep_alive
        } else {
            !conn_close
        },
    })
}

/// Reads one request-head line of at most `room` bytes into `buf`, returning its
/// length. `Ok(None)` means the line is longer than `room`: it is never buffered
/// past the cap (E2).
pub(super) fn head_line_into<R: Read>(
    reader: &mut BufReader<R>,
    room: usize,
    buf: &mut Vec<u8>,
) -> std::io::Result<Option<usize>> {
    buf.clear();
    let n = (&mut *reader).take(room as u64 + 1).read_until(b'\n', buf)?;
    if n > room {
        return Ok(None);
    }
    Ok(Some(n))
}

/// Answers an over-long request head with 431 at once, then reads and
/// discards what the client still sends until it closes (or the read
/// deadline passes) so the answer is not reset by the socket (E2).
pub(super) fn refuse_head<S: SockStream>(reader: &mut BufReader<S>) -> Option<Request> {
    let body = format!(
        "{{\"error\": \"request head too large: the limit is {MAX_HEAD_BYTES} bytes\"}}"
    );
    let response = format!(
        "HTTP/1.1 431 Request Header Fields Too Large\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    {
        let s = reader.get_mut();
        let _ = s.write_all(response.as_bytes());
        let _ = s.flush();
    }
    let _ = reader.get_ref().set_rt(std::time::Duration::from_millis(200));
    let mut sink = [0u8; 8192];
    while matches!(reader.read(&mut sink), Ok(n) if n > 0) {}
    None
}

/// The note pushed into a thread when a delete is parked (EB1, ER1).
pub fn parked_note(path: &str) -> String {
    format!("Note: {path} has NOT been deleted yet; the deletion is waiting for your approval in Decisions.")
}

/// Whether a Host header names this machine (DNS-rebinding guard).
pub(super) fn host_name(host: &str) -> String {
    match host.strip_prefix('[') {
        Some(rest) => format!("[{}]", rest.split(']').next().unwrap_or("")).to_ascii_lowercase(),
        None => host.split(':').next().unwrap_or("").to_ascii_lowercase(),
    }
}

/// Whether a Host names this machine or one the operator allowed (EH1).
#[inline]
pub(crate) fn host_allowed(host: &str, allowed: &[String]) -> bool {
    let name = host_name(host);
    matches!(name.as_str(), "localhost" | "127.0.0.1" | "[::1]")
        || allowed.iter().any(|a| a.eq_ignore_ascii_case(&name))
}

pub(super) fn ct_eq(a: &str, b: &str) -> bool {
    let (ab, bb) = (a.as_bytes(), b.as_bytes());
    if ab.len() != bb.len() {
        return false;
    }
    let mut diff = 0u8;
    for (&x, &y) in ab.iter().zip(bb.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

pub(super) fn bearer_token(req: &Request) -> Option<&str> {
    let raw = req.authorization.as_deref()?.trim();
    if raw.len() < 7 || !raw[..7].eq_ignore_ascii_case("bearer ") {
        return None;
    }
    let tok = raw[7..].trim();
    if tok.is_empty() { None } else { Some(tok) }
}

pub(super) fn is_admin_role_label(role: &str) -> bool {
    matches!(
        role.trim().to_ascii_lowercase().as_str(),
        "admin" | "administrator" | "owner" | "it_admin" | "it-admin" | "it admin" | "operator" | "superuser"
    )
}

/// Checks whether a request authenticates past `ERGON_GATE_TOKEN` (when configured).
/// Accepts `X-Ergon-Gate`, `Authorization: Bearer`, `X-Ergon-Token`, or a matching
/// dedicated `ERGON_ADMIN_TOKEN`. If `X-Ergon-Gate` is present, it must match.
pub(super) fn gate_authenticated(state: &AppState, req: &Request) -> bool {
    let Some(expected) = &state.gate_token else {
        return true;
    };
    if let Some(g) = req.gate.as_deref() {
        return ct_eq(g, expected);
    }
    let bearer = bearer_token(req);
    let x_token = req.api_token.as_deref();
    if bearer.is_some_and(|t| ct_eq(t, expected)) || x_token.is_some_and(|t| ct_eq(t, expected)) {
        return true;
    }
    if let Some(adm) = &state.admin_token {
        let x_admin = req.admin_token.as_deref();
        if bearer.is_some_and(|t| ct_eq(t, adm))
            || x_token.is_some_and(|t| ct_eq(t, adm))
            || x_admin.is_some_and(|t| ct_eq(t, adm))
        {
            return true;
        }
    }
    false
}

/// Update management is admin-only: in multi-user auth mode an authenticated
/// `Role::Admin` principal (including audited break-glass) qualifies; otherwise
/// fall back to the hosting-layer token / loopback rules in `is_update_admin`.
pub(super) fn update_admin_ok(
    state: &AppState,
    req: &Request,
    principal: Option<&auth::AuthPrincipal>,
    auth_enforced: bool,
) -> bool {
    if auth_enforced && principal.is_some_and(|p| p.role == auth::Role::Admin) {
        return true;
    }
    is_update_admin(state, req)
}

/// Determines whether the caller is authorized to manage updates (`check`, `mode`,
/// `install`/`apply`, `restart`) on this Ergon instance:
/// - Explicit non-admin `X-Ergon-Role` always denies (`403`).
/// - Dedicated `ERGON_ADMIN_TOKEN`, if set, must match.
/// - Behind `ERGON_GATE_TOKEN`, allows ONLY authenticated admins presenting
///   `Authorization: Bearer` / `X-Ergon-Token` / `X-Ergon-Admin-Token` matching the gate token,
///   or a trusted hosting proxy's `X-Ergon-Role: admin` header. Never trusts editable
///   `person.role` fields or standalone client-supplied `X-Ergon-Admin` headers.
/// - Without `ERGON_GATE_TOKEN`, allows loopback local admins and refuses non-loopback LAN hosts.
pub(super) fn is_update_admin(state: &AppState, req: &Request) -> bool {
    let bearer = bearer_token(req);
    let x_token = req.api_token.as_deref();
    let x_admin_token = req.admin_token.as_deref();

    if let Some(role) = req.actor_role.as_deref().map(str::trim).filter(|r| !r.is_empty()) {
        if !is_admin_role_label(role) {
            return false;
        }
    }

    if let Some(adm) = &state.admin_token {
        return x_admin_token.is_some_and(|t| ct_eq(t, adm))
            || bearer.is_some_and(|t| ct_eq(t, adm))
            || x_token.is_some_and(|t| ct_eq(t, adm));
    }

    if let Some(gate_expected) = &state.gate_token {
        if x_admin_token.is_some_and(|t| !ct_eq(t, gate_expected))
            || x_token.is_some_and(|t| !ct_eq(t, gate_expected))
            || bearer.is_some_and(|t| !ct_eq(t, gate_expected))
        {
            return false;
        }
        let token_admin = bearer.is_some_and(|t| ct_eq(t, gate_expected))
            || x_token.is_some_and(|t| ct_eq(t, gate_expected))
            || x_admin_token.is_some_and(|t| ct_eq(t, gate_expected));
        let role_admin = req
            .actor_role
            .as_deref()
            .map(str::trim)
            .is_some_and(is_admin_role_label);
        return token_admin || role_admin;
    }

    if x_admin_token.is_some() {
        return false;
    }
    host_allowed(req.host.as_deref().unwrap_or(""), &[])
}

/// Refuses a request a page on another site could have sent (E1), with
/// the same observable rules as the Fleet app: a foreign Host (unless
/// the hosting gate is set), a state-changing request with a body that
/// is not JSON (a text/plain POST has no preflight), cross-site fetch
/// metadata, and an Origin whose authority is not the request's Host.
pub(crate) fn refusal(req: &Request, gated: bool, allowed: &[String]) -> Option<(u16, &'static str)> {
    if !gated && !host_allowed(req.host.as_deref().unwrap_or(""), allowed) {
        return Some((403, "requests must address this machine as localhost, 127.0.0.1 or [::1], or a host listed in ERGON_ALLOWED_HOSTS"));
    }
    if !matches!(req.method.as_str(), "POST" | "PUT" | "PATCH" | "DELETE") {
        return None;
    }
    if !req.body.is_empty() && !req.content_type.starts_with("application/json") {
        return Some((415, "POST requires Content-Type: application/json"));
    }
    if req.sec_fetch_site.as_deref() == Some("cross-site") {
        return Some((403, "cross-site requests are refused"));
    }
    if let Some(origin) = &req.origin {
        let authority = origin.split("://").nth(1).unwrap_or("").split('/').next().unwrap_or("");
        let matches = req
            .host
            .as_deref()
            .map(|h| h.eq_ignore_ascii_case(authority))
            .unwrap_or(false);
        if !matches {
            return Some((403, "cross-origin requests are refused"));
        }
    }
    None
}

pub(super) fn respond(
    stream: &mut ConnStream,
    status: u16,
    content_type: &str,
    body: &str,
) -> std::io::Result<()> {
    respond_h(stream, status, content_type, body, &[])
}

#[inline]
pub(super) fn respond_bytes(
    stream: &mut ConnStream,
    status: u16,
    content_type: &str,
    body: &[u8],
    extra_headers: &[(&str, String)],
) -> std::io::Result<()> {
    respond_bytes_inner(stream, status, content_type, body, extra_headers, false)
}

pub(super) fn respond_head(
    stream: &mut ConnStream,
    status: u16,
    content_type: &str,
    content_length: usize,
    extra_headers: &[(&str, String)],
) -> std::io::Result<()> {
    use std::fmt::Write as _;
    let reason = status_reason(status);
    let keep_open = KEEP_ALIVE_CONN.get() && status < 400 && !shutdown_requested();
    if !keep_open {
        KEEP_ALIVE_CONN.set(false);
    }
    let conn_hdr = if keep_open { "keep-alive" } else { "close" };
    let mut head = String::with_capacity(384);
    let _ = write!(
        &mut head,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {content_length}\r\nConnection: {conn_hdr}\r\n"
    );
    if keep_open {
        head.push_str("Keep-Alive: timeout=2, max=10000\r\n");
    }
    for (name, value) in extra_headers {
        let _ = write!(&mut head, "{name}: {value}\r\n");
    }
    let has = |n: &str| extra_headers.iter().any(|(h, _)| h.eq_ignore_ascii_case(n));
    for (name, value) in tripwire::BASE_HEADERS {
        if !has(name) {
            let _ = write!(&mut head, "{name}: {value}\r\n");
        }
    }
    if TLS_CONN.get() && !has("Strict-Transport-Security") {
        head.push_str("Strict-Transport-Security: max-age=31536000; includeSubDomains\r\n");
    }
    if content_type.starts_with("application/json") && !has("Cache-Control") {
        head.push_str("Cache-Control: no-store\r\n");
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes())?;
    stream.flush()
}

pub(super) fn status_reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        413 => "Content Too Large",
        415 => "Unsupported Media Type",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "OK",
    }
}

#[inline]
pub(super) fn respond_bytes_inner(
    stream: &mut ConnStream,
    status: u16,
    content_type: &str,
    body: &[u8],
    extra_headers: &[(&str, String)],
    omit_body: bool,
) -> std::io::Result<()> {
    use std::fmt::Write as _;
    let reason = status_reason(status);
    let body_len = if status == 304 { 0 } else { body.len() };
    let keep_open = KEEP_ALIVE_CONN.get() && status < 400 && !shutdown_requested();
    if !keep_open {
        KEEP_ALIVE_CONN.set(false);
    }
    let conn_hdr = if keep_open { "keep-alive" } else { "close" };
    let mut head = String::with_capacity(384);
    if status == 304 {
        let _ = write!(&mut head, "HTTP/1.1 304 {reason}\r\nConnection: {conn_hdr}\r\n");
    } else {
        let _ = write!(
            &mut head,
            "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {body_len}\r\nConnection: {conn_hdr}\r\n"
        );
    }
    if keep_open {
        head.push_str("Keep-Alive: timeout=2, max=10000\r\n");
    }
    for (name, value) in extra_headers {
        let _ = write!(&mut head, "{name}: {value}\r\n");
    }
    // Hardening on every response (unless the caller set its own).
    if extra_headers.is_empty() {
        head.push_str(tripwire::BASE_HEADERS_BLOCK);
        if TLS_CONN.get() {
            head.push_str("Strict-Transport-Security: max-age=31536000; includeSubDomains\r\n");
        }
        if content_type.starts_with("application/json") {
            head.push_str("Cache-Control: no-store\r\n");
        }
    } else {
        let has = |n: &str| extra_headers.iter().any(|(h, _)| h.eq_ignore_ascii_case(n));
        for (name, value) in tripwire::BASE_HEADERS {
            if !has(name) {
                let _ = write!(&mut head, "{name}: {value}\r\n");
            }
        }
        if TLS_CONN.get() && !has("Strict-Transport-Security") {
            head.push_str("Strict-Transport-Security: max-age=31536000; includeSubDomains\r\n");
        }
        if content_type.starts_with("application/json") && !has("Cache-Control") {
            head.push_str("Cache-Control: no-store\r\n");
        }
    }
    head.push_str("\r\n");
    if omit_body || status == 304 || body.is_empty() {
        stream.write_all(head.as_bytes())?;
        return stream.flush();
    }
    match stream {
        ConnStream::Plain(tcp) => {
            let mut bufs = [
                std::io::IoSlice::new(head.as_bytes()),
                std::io::IoSlice::new(body),
            ];
            let mut slice_ref: &mut [std::io::IoSlice<'_>] = &mut bufs;
            while !slice_ref.is_empty() {
                let n = tcp.write_vectored(slice_ref)?;
                if n == 0 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::WriteZero,
                        "write_vectored 0",
                    ));
                }
                std::io::IoSlice::advance_slices(&mut slice_ref, n);
            }
            Ok(())
        }
        ConnStream::Tls(_) => {
            stream.write_all(head.as_bytes())?;
            stream.write_all(body)?;
            stream.flush()
        }
    }
}

pub(super) fn respond_h(
    stream: &mut ConnStream,
    status: u16,
    content_type: &str,
    body: &str,
    extra_headers: &[(&str, String)],
) -> std::io::Result<()> {
    respond_bytes(stream, status, content_type, body.as_bytes(), extra_headers)
}

pub(super) fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hex = &s[i + 1..i + 3];
                if let Ok(v) = u8::from_str_radix(hex, 16) {
                    out.push(v);
                    i += 3;
                } else {
                    out.push(b'%');
                    i += 1;
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub(super) fn query_param(query: &str, key: &str) -> Option<String> {
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        if percent_decode(k) == key {
            return Some(percent_decode(v));
        }
    }
    None
}

pub(super) fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0u8;
    for b in input.bytes() {
        if b.is_ascii_whitespace() {
            continue;
        }
        if b == b'=' {
            break;
        }
        let val = match b {
            b'A'..=b'Z' => (b - b'A') as u32,
            b'a'..=b'z' => (b - b'a' + 26) as u32,
            b'0'..=b'9' => (b - b'0' + 52) as u32,
            b'+' => 62,
            b'/' => 63,
            _ => return Err("invalid base64 data".to_string()),
        };
        buf = (buf << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xff) as u8);
        }
    }
    Ok(out)
}

pub(super) fn mime_for_ext(ext: &str) -> &'static str {
    match ext {
        "pdf" => "application/pdf",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "csv" => "text/csv; charset=utf-8",
        "md" | "markdown" => "text/markdown; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "txt" | "log" | "rs" | "ts" | "tsx" | "js" | "py" | "toml" | "yaml" | "yml" => {
            "text/plain; charset=utf-8"
        }
        _ => "application/octet-stream",
    }
}

pub(super) fn effective_cfg(state: &AppState, runtime: &PersonRuntime) -> Config {
    let mut cfg = runtime.cfg.clone();
    let live = state.model_cfg.lock().unwrap();
    cfg.model_base_url = live.model_base_url.clone();
    cfg.model_id = live.model_id.clone();
    cfg.api_key = live.api_key.clone();
    cfg
}

pub(super) const VERSION_JSON_BYTES: &[u8] = concat!(
    r#"{"name":"ergon","version":""#,
    env!("CARGO_PKG_VERSION"),
    r#""}"#,
)
.as_bytes();

#[inline]
pub(super) fn respond_json_with<F: FnOnce(&mut Vec<u8>)>(
    stream: &mut ConnStream,
    status: u16,
    fill: F,
) -> std::io::Result<()> {
    JSON_SCRATCH.with(|cell| {
        let mut buf = cell.borrow_mut();
        buf.clear();
        fill(&mut buf);
        let res = respond_bytes(stream, status, "application/json", &buf, &[]);
        if buf.capacity() > 32 * 1024 {
            *buf = Vec::with_capacity(4096);
        }
        res
    })
}

#[inline(never)]
pub(super) fn person_idx(state: &AppState, person: &Person, runtime: &PersonRuntime) -> Arc<search::PersonIndex> {
    state.search_cache.get(
        &person.id,
        &runtime.cfg.data_dir,
        &runtime.cfg.workspace_dir,
        &runtime.cfg.memory_path,
    )
}

#[cold]
#[inline(never)]
pub(super) fn respond_err(stream: &mut ConnStream, status: u16, msg: &str) -> std::io::Result<()> {
    respond_json_with(stream, status, |buf| {
        buf.extend_from_slice(br#"{"error":"#);
        search::write_json_str(buf, msg);
        buf.push(b'}');
    })
}

pub(super) fn respond_json(stream: &mut ConnStream, status: u16, v: &Value) -> std::io::Result<()> {
    JSON_SCRATCH.with(|cell| {
        let mut buf = cell.borrow_mut();
        buf.clear();
        if serde_json::to_writer(&mut *buf, v).is_err() {
            buf.clear();
            buf.extend_from_slice(v.to_string().as_bytes());
        }
        let res = respond_bytes(stream, status, "application/json", &buf, &[]);
        if buf.capacity() > 32 * 1024 {
            *buf = Vec::with_capacity(4096);
        }
        res
    })
}

/// Markdown for the audit bundle's tripwire section.
pub(super) fn render_security_markdown(sec: &Value) -> String {
    let mut md = String::from("\n\n## Security tripwires\n\n");
    md.push_str(&format!(
        "- Log chain: **{}** ({} entries, head `{}`)\n- Events shown for this person: {} (SHA-256 `{}`)\n\n",
        if sec["log_verified"].as_bool().unwrap_or(false) { "verified" } else { "BROKEN — edited outside Ergon" },
        sec["log_length"],
        sec["log_head_hash"].as_str().unwrap_or(""),
        sec["events_for_person"],
        sec["events_sha256"].as_str().unwrap_or(""),
    ));
    if let Some(events) = sec["events"].as_array() {
        if events.is_empty() {
            md.push_str("No tripwire has fired.\n");
        }
        for e in events.iter().take(100) {
            md.push_str(&format!(
                "- {} · `{}` · {} · {} — {}\n",
                records::date_string(e["ts"].as_u64().unwrap_or(0)),
                e["code"].as_str().unwrap_or(""),
                e["severity"].as_str().unwrap_or(""),
                e["title"].as_str().unwrap_or(""),
                e["action"].as_str().unwrap_or(""),
            ));
        }
    }
    md
}

/// Carry out an approved (or declined) tripwire decision and return the
/// updated decision + outcome. `None` for decisions that are not the
/// tripwire's (deletes and questions keep their own handling).
pub(super) fn settle_tripwire_decision(
    state: &AppState,
    runtime: &PersonRuntime,
    person: &Person,
    decision: &Value,
    approve: bool,
) -> Option<(Value, String)> {
    let kind = decision["kind"].as_str()?;
    let id = decision["id"].as_str()?;
    let root = &state.cfg.data_dir;
    let outcome = match (kind, approve) {
        ("security_notice", _) => "Noted — Ergon keeps the traps armed.".to_string(),
        ("security", false) => "Dismissed — nothing else was changed.".to_string(),
        ("security", true) => {
            let value = decision["source"]["value"].as_str().unwrap_or("");
            match decision["source"]["kind"].as_str() {
                Some("url") | Some("host") => {
                    let host = tripwire::normalize_host(value);
                    match tripwire::update_policy(root, "block", &host, true) {
                        Ok(_) => format!("Blocked {host}: your agent will not open it again (Settings → Security can undo this)."),
                        Err(e) => format!("Could not block that site: {e}"),
                    }
                }
                Some("file") => quarantine_file(&runtime.cfg, value),
                _ => "Noted — nothing else needed doing.".to_string(),
            }
        }
        ("allow_host", true) => {
            let host = decision["path"].as_str().unwrap_or("");
            match tripwire::update_policy(root, "allow", host, true) {
                Ok(_) => format!("Allowed {host}: future tasks can open it (Settings → Security can undo this)."),
                Err(e) => format!("Could not allow it: {e}"),
            }
        }
        ("allow_host", false) => "Kept blocked.".to_string(),
        ("memory_note", true) => {
            let note = decision["note"].as_str().unwrap_or("");
            let cfg = effective_cfg(state, runtime);
            match skills::update_memory(&cfg, note) {
                Ok(_) => "Saved to your agent's memory.".to_string(),
                Err(e) => format!("Approved, but saving failed: {e}"),
            }
        }
        ("memory_note", false) => "Discarded — your agent's memory is unchanged.".to_string(),
        ("publish_skill", true) => {
            let p = &decision["payload"];
            let source = p["source"].as_str().unwrap_or("");
            let params = string_list(&p["params"]);
            match skills::check_script(&runtime.cfg, source) {
                Err(e) => format!("Approved, but the skill no longer builds: {e}"),
                Ok(()) => {
                    let mut lib = state.library.lock().unwrap();
                    match lib.publish_skill(
                        p["name"].as_str().unwrap_or("skill"),
                        p["description"].as_str().unwrap_or(""),
                        params,
                        source.to_string(),
                        &person.id,
                    ) {
                        Ok(item) => {
                            lib.save(root);
                            format!("Published \"{}\" to the company library.", item.name)
                        }
                        Err(e) => format!("Approved, but publishing failed: {e}"),
                    }
                }
            }
        }
        ("publish_skill", false) => "Not published — it stays out of the library.".to_string(),
        _ => return None,
    };
    let updated = records::set_decision_outcome(&runtime.cfg.data_dir, id, &outcome)
        .unwrap_or_else(|| decision.clone());
    Some((updated, outcome))
}

/// Move a workspace file the tripwire flagged out of the agent's reach
/// (into the person's data dir, never deleted), with a history snapshot.
pub(super) fn quarantine_file(cfg: &Config, rel: &str) -> String {
    let full = match sandbox_path_resolving(&cfg.workspace_dir, rel, false) {
        Ok(p) => p,
        Err(e) => return format!("Could not quarantine {rel}: {e}"),
    };
    match full.symlink_metadata() {
        Ok(md) if md.is_file() || md.file_type().is_symlink() => {}
        _ => return format!("{rel} is no longer in the workspace — nothing to quarantine."),
    }
    let _ = file_history::record_revision(&cfg.data_dir, &cfg.workspace_dir, rel, "quarantine");
    let dir = cfg.data_dir.join("quarantine");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        return format!("Could not quarantine {rel}: {e}");
    }
    let base = full
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".to_string());
    let dest = dir.join(format!("{}-{base}", records::now_millis()));
    match std::fs::rename(&full, &dest) {
        Ok(()) => format!("Quarantined {rel}: moved out of your agent's reach into your Ergon data folder (quarantine/), kept intact."),
        Err(e) => format!("Could not quarantine {rel}: {e}"),
    }
}

/// A run the tripwire stopped is not a server error: the thread says
/// plainly what happened and where the details are, and the response is
/// the thread (200) marked `stopped_for_safety`.
pub(super) fn respond_safety_stop(
    stream: &mut ConnStream,
    runtime: &PersonRuntime,
    thread_id: &str,
    err: &str,
) -> std::io::Result<()> {
    let reason = err.trim_start_matches(tripwire::HALT_PREFIX);
    let mut store = runtime.store.lock().unwrap();
    store.push(
        thread_id,
        "step",
        &format!("Ergon stopped this task to keep your work safe: {reason}. Nothing further was run or sent. The details and one-tap follow-ups are waiting in Today."),
    );
    let mut v = store
        .thread_json(thread_id)
        .unwrap_or_else(|| json!({"thread_id": thread_id}));
    v["stopped_for_safety"] = json!(true);
    respond_json(stream, 200, &v)
}

/// The CSP for the embedded desk page, computed once from its own inline
/// scripts (see `tripwire::csp_for_html`).
pub(super) fn page_csp(page: &'static str) -> String {
    static INDEX: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    INDEX.get_or_init(|| tripwire::csp_for_html(page)).clone()
}

pub(super) fn accepts_gzip(header: Option<&str>) -> bool {
    let Some(raw) = header else { return false };
    for part in raw.split(',') {
        let mut pieces = part.trim().split(';');
        let enc = pieces.next().unwrap_or("").trim();
        if !enc.eq_ignore_ascii_case("gzip") {
            continue;
        }
        let mut rejected = false;
        for param in pieces {
            let param = param.trim();
            if let Some(q) = param.strip_prefix("q=").or_else(|| param.strip_prefix("Q=")) {
                if q.trim().parse::<f32>().ok().is_some_and(|v| v <= 0.0) {
                    rejected = true;
                }
            }
        }
        if !rejected {
            return true;
        }
    }
    false
}

pub(super) fn etag_matches(if_none_match: Option<&str>, etag: &str) -> bool {
    let Some(raw) = if_none_match else { return false };
    for item in raw.split(',') {
        let tag = item.trim();
        if tag == "*" || tag == etag || tag.strip_prefix("W/") == Some(etag) {
            return true;
        }
    }
    false
}

pub(super) fn serve_chunk(stream: &mut ConnStream, req: &Request, path: &str) -> std::io::Result<()> {
    let Some(chunk) = ui::chunk(path) else {
        return respond_err(stream, 404, "chunk not found");
    };
    let wants_gzip = chunk.pre_gzipped && accepts_gzip(req.accept_encoding.as_deref());
    let etag = if wants_gzip { chunk.etag_gzip } else { chunk.etag_raw };
    let mut headers: Vec<(&str, String)> = vec![
        ("ETag", etag.to_string()),
        ("Vary", "Accept-Encoding".to_string()),
        ("Cache-Control", "public, max-age=31536000, immutable".to_string()),
    ];
    if etag_matches(req.if_none_match.as_deref(), etag) {
        return respond_bytes(stream, 304, chunk.mime, &[], &headers);
    }
    let omit_body = req.method == "HEAD";
    if wants_gzip {
        headers.push(("Content-Encoding", "gzip".to_string()));
        respond_bytes_inner(stream, 200, chunk.mime, chunk.bytes, &headers, omit_body)
    } else {
        let raw = ui::chunk_raw(chunk);
        respond_bytes_inner(stream, 200, chunk.mime, raw.as_ref(), &headers, omit_body)
    }
}

/// Serve a desk page (`/` or `/phone`). When multi-user auth is enforced and
/// the caller is not yet signed in, no session cookie is minted (the SPA renders
/// the sign-in / first-admin setup screen and all `/api/*` data endpoints remain
/// gated with 401). In legacy single-user loopback mode, preserves existing
/// per-browser session cookie behavior.
pub(super) fn serve_page(
    stream: &mut ConnStream,
    state: &AppState,
    req: &Request,
    person: &Person,
    page: &'static str,
    auth_enforced: bool,
) -> std::io::Result<()> {
    let bundle = state.tripwire.decorate_html_bundle(page);
    let wants_gzip = accepts_gzip(req.accept_encoding.as_deref());
    let etag = if wants_gzip { &bundle.etag_gzip } else { &bundle.etag_raw };
    let mut headers: Vec<(&str, String)> = vec![
        ("Content-Security-Policy", page_csp(page)),
        (
            "Content-Security-Policy-Report-Only",
            "require-trusted-types-for 'script'".to_string(),
        ),
        ("ETag", etag.clone()),
        ("Vary", "Accept-Encoding".to_string()),
        ("Cache-Control", "no-cache".to_string()),
    ];
    if !auth_enforced && !session_known(state, &req.session) {
        let sid = new_session_id();
        insert_session(state, sid.clone(), person.id.clone());
        let secure = if state.tls.is_some() { "; Secure" } else { "" };
        headers.push((
            "Set-Cookie",
            format!("ergon_session={sid}; Path=/; HttpOnly; SameSite=Lax{secure}"),
        ));
        headers.push((
            "Set-Cookie",
            format!("ergon_role={}; Path=/; SameSite=Strict", tripwire::ROLE_COOKIE_VALUE),
        ));
    }
    if etag_matches(req.if_none_match.as_deref(), etag) {
        return respond_bytes_inner(stream, 304, "text/html; charset=utf-8", &[], &headers, true);
    }
    let omit_body = req.method == "HEAD";
    if wants_gzip {
        headers.push(("Content-Encoding", "gzip".to_string()));
        respond_bytes_inner(stream, 200, "text/html; charset=utf-8", &bundle.gzip, &headers, omit_body)
    } else {
        respond_bytes_inner(
            stream,
            200,
            "text/html; charset=utf-8",
            bundle.html.as_bytes(),
            &headers,
            omit_body,
        )
    }
}

pub(super) fn handle_webhook_hook_request(
    stream: &mut ConnStream,
    state: &Arc<AppState>,
    req: &Request,
) -> std::io::Result<()> {
    let hook_id = req.path.trim_start_matches("/api/hooks/").trim();
    if hook_id.is_empty() || hook_id.contains('/') || hook_id.contains("..") {
        return respond_err(stream, 401, "invalid or missing webhook authentication");
    }
    let people_snapshot = state.people.read().unwrap().people.clone();
    let mut ordered: Vec<Person> = Vec::with_capacity(people_snapshot.len());
    let active = effective_person(state, &req.session);
    ordered.push(active.clone());
    for p in people_snapshot {
        if p.id != active.id {
            ordered.push(p);
        }
    }
    for person in ordered {
        let rt = runtime_for(state, &person);
        match connectors::verify_and_ingest_webhook_with_meta(
            &rt.cfg,
            hook_id,
            req.hook_signature.as_deref(),
            req.hook_token.as_deref(),
            req.hook_timestamp.as_deref(),
            req.hook_delivery.as_deref(),
            &req.content_type,
            &req.body,
        ) {
            Ok(ingested) => {
                state.search_cache.invalidate(&person.id);
                let run_info = fire_linked_trigger_target(
                    state,
                    &person,
                    &rt,
                    ingested.routine_id.as_deref(),
                    &ingested.target_kind,
                    &ingested.trigger_file,
                    "webhook",
                );
                return respond_json(
                    stream,
                    202,
                    &json!({
                        "accepted": true,
                        "trigger_id": ingested.trigger_id,
                        "trigger_name": ingested.trigger_name,
                        "trigger_file": ingested.trigger_file,
                        "routine_id": ingested.routine_id,
                        "received_ms": ingested.received_ms,
                        "run": run_info,
                    }),
                );
            }
            Err((401, _)) => continue,
            Err((status, msg)) => return respond_err(stream, status, &msg),
        }
    }
    respond_err(stream, 401, "invalid or missing webhook authentication")
}

pub(super) fn mcp_inbound_bearer_valid(authorization: Option<&str>) -> bool {
    let Ok(expected) = std::env::var("ERGON_MCP_INBOUND_TOKEN") else {
        return false;
    };
    let expected = expected.trim();
    if expected.is_empty() {
        return false;
    }
    let Some(presented) = auth::extract_bearer_token(authorization) else {
        return false;
    };
    let h1 = tripwire::sha256(presented.as_bytes());
    let h2 = tripwire::sha256(expected.as_bytes());
    tripwire::ct_eq(&h1, &h2)
}

pub(super) fn enforce_api_key_scope(p: &auth::AuthPrincipal, method: &str, path: &str) -> Result<(), String> {
    if p.api_key_scopes.is_none() {
        return Ok(());
    }
    if matches!(
        (method, path),
        ("GET" | "HEAD", "/" | "/phone" | "/api/health")
            | ("GET", "/api/auth/me")
            | ("POST", "/api/auth/login" | "/api/auth/logout")
    ) || path.starts_with("/chunks/")
    {
        return Ok(());
    }

    if path == "/api/keys"
        || path.starts_with("/api/keys/")
        || path == "/api/auth/users"
        || path.starts_with("/api/auth/users/")
        || path == "/api/auth/oidc/config"
        || path == "/api/config"
        || path == "/api/people"
        || path == "/api/connectors"
        || path.starts_with("/api/connectors/")
        || path == "/api/mcp/servers"
        || path.starts_with("/api/mcp/servers/")
        || path == "/api/mcp/inbound/config"
        || matches!(
            path,
            "/api/update/check" | "/api/update/apply" | "/api/update/restart" | "/api/security" | "/api/security/policy"
        )
    {
        return if p.has_scope("admin") {
            Ok(())
        } else {
            Err("forbidden: API key lacks required scope 'admin'".to_string())
        };
    }

    if method == "POST" && path == "/api/mcp/inbound" {
        return if p.has_scope("mcp") {
            Ok(())
        } else {
            Err("forbidden: API key lacks required scope 'mcp'".to_string())
        };
    }

    if path == "/api/webhooks/subscriptions" || path.starts_with("/api/webhooks/subscriptions/") {
        return if p.has_scope("webhooks") {
            Ok(())
        } else {
            Err("forbidden: API key lacks required scope 'webhooks'".to_string())
        };
    }

    if path == "/api/company/memory" || path.starts_with("/api/company/memory/") {
        return if method == "GET" {
            if p.has_scope("read") || p.has_scope("company_db") {
                Ok(())
            } else {
                Err("forbidden: API key lacks required scope 'read' or 'company_db'".to_string())
            }
        } else if p.has_scope("company_db") || p.has_scope("write") {
            Ok(())
        } else {
            Err("forbidden: API key lacks required scope 'company_db' or 'write'".to_string())
        };
    }

    if path == "/api/company/db" || path == "/api/company/db/query" {
        return if p.has_scope("company_db") {
            Ok(())
        } else {
            Err("forbidden: API key lacks required scope 'company_db'".to_string())
        };
    }

    if path == "/api/tools" {
        return if p.has_scope("tools") || p.has_scope("read") {
            Ok(())
        } else {
            Err("forbidden: API key lacks required scope 'tools' or 'read'".to_string())
        };
    }

    if path == "/api/tools/test"
        || path == "/api/tools/build"
        || (path.starts_with("/api/tools/") && path.ends_with("/run"))
    {
        return if p.has_scope("tools") {
            Ok(())
        } else {
            Err("forbidden: API key lacks required scope 'tools'".to_string())
        };
    }

    if matches!(method, "GET" | "HEAD") {
        if p.has_scope("read") {
            Ok(())
        } else {
            Err("forbidden: API key lacks required scope 'read'".to_string())
        }
    } else if p.has_scope("write") {
        Ok(())
    } else {
        Err("forbidden: API key lacks required scope 'write'".to_string())
    }
}

pub(super) fn person_by_id(state: &AppState, id: &str) -> Person {
    let people = state.people.read().unwrap();
    if let Some(p) = people.people.iter().find(|p| p.id == id) {
        return p.clone();
    }
    people.active_person()
}

