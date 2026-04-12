use std::collections::HashSet;
use std::io::{self, BufRead, Write};
use std::time::Duration;

use chrono::Utc;
use clap::Parser;
use imap::Session;
use native_tls::TlsStream;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// --- CLI ---

#[derive(Clone, Parser)]
#[command(name = "imap-watcher")]
struct Cli {
    /// IMAP server hostname
    #[arg(long)]
    server: String,

    /// IMAP server port
    #[arg(long, default_value_t = 993)]
    port: u16,

    /// Username
    #[arg(long)]
    user: String,

    /// Password (prefer --password-env for security)
    #[arg(long, required_unless_present = "password_env")]
    password: Option<String>,

    /// Environment variable name containing the password
    #[arg(long)]
    password_env: Option<String>,

    /// IMAP folder to watch
    #[arg(long, default_value = "INBOX")]
    folder: String,

    /// Polling interval in seconds (fallback when IDLE is not supported)
    #[arg(long, default_value_t = 60)]
    poll_interval: u64,

    /// Use TLS
    #[arg(long, default_value_t = true)]
    tls: bool,

    /// Regex filter for subject (case-insensitive)
    #[arg(long)]
    subject_match: Option<String>,

    /// Regex filter for body (case-insensitive)
    #[arg(long)]
    body_match: Option<String>,
}

// --- JSON-RPC types ---

#[derive(Deserialize)]
struct JsonRpcMessage {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[allow(dead_code)]
    #[serde(default)]
    params: Value,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    result: Value,
}

#[derive(Serialize)]
struct JsonRpcNotification {
    jsonrpc: String,
    method: String,
    params: Value,
}

// --- Helpers ---

fn send_json(value: &impl Serialize) {
    let line = serde_json::to_string(value).expect("serialize");
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let _ = out.write_all(line.as_bytes());
    let _ = out.write_all(b"\n");
    let _ = out.flush();
}

fn send_response(id: Value, result: Value) {
    send_json(&JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result,
    });
}

fn send_event(event_type: &str, data: Value) {
    send_json(&JsonRpcNotification {
        jsonrpc: "2.0".to_string(),
        method: "event".to_string(),
        params: serde_json::json!({
            "eventType": event_type,
            "data": data,
        }),
    });
}

fn compile_filter(pattern: &Option<String>) -> Option<Regex> {
    pattern.as_ref().and_then(|p| {
        let case_insensitive = format!("(?i){}", p);
        Regex::new(&case_insensitive).ok()
    })
}

fn matches_filters(
    subject: &str,
    body_text: &str,
    body_html: &str,
    subject_re: &Option<Regex>,
    body_re: &Option<Regex>,
) -> bool {
    let subject_ok = subject_re.as_ref().is_none_or(|re| re.is_match(subject));
    let body_ok = body_re.as_ref().is_none_or(|re| {
        re.is_match(body_text)
            || (!body_html.is_empty() && re.is_match(&strip_html_tags(body_html)))
    });
    subject_ok && body_ok
}

fn strip_html_tags(html: &str) -> String {
    let tag_re = Regex::new(r"<[^>]+>").unwrap();
    let text = tag_re.replace_all(html, "");
    let entity_re = Regex::new(r"&(?:#\d+|#x[0-9a-fA-F]+|\w+);").unwrap();
    entity_re.replace_all(&text, " ").trim().to_string()
}

// --- Mail parsing ---

fn get_header(headers: &[mailparse::MailHeader], name: &str) -> String {
    headers
        .iter()
        .find(|h| h.get_key_ref().eq_ignore_ascii_case(name))
        .map(|h| h.get_value())
        .unwrap_or_default()
}

fn extract_mail_data(raw: &[u8]) -> Option<Value> {
    let parsed = mailparse::parse_mail(raw).ok()?;
    let headers = &parsed.headers;

    let subject = get_header(headers, "subject");
    let from = get_header(headers, "from");
    let date = get_header(headers, "date");
    let message_id = get_header(headers, "message-id");

    let (body_text, body_html) = extract_body_parts(&parsed);

    Some(serde_json::json!({
        "message_id": message_id,
        "from": from,
        "subject": subject,
        "date": date,
        "body_text": body_text,
        "body_html": body_html,
        "timestamp": Utc::now().to_rfc3339(),
    }))
}

fn extract_body_parts(parsed: &mailparse::ParsedMail) -> (String, String) {
    let mut text = String::new();
    let mut html = String::new();

    if parsed.subparts.is_empty() {
        let content_type = parsed.ctype.mimetype.to_lowercase();
        if let Ok(body) = parsed.get_body() {
            if content_type.contains("text/plain") {
                text = body;
            } else if content_type.contains("text/html") {
                html = body;
            }
        }
    } else {
        for part in &parsed.subparts {
            let (t, h) = extract_body_parts(part);
            if text.is_empty() && !t.is_empty() {
                text = t;
            }
            if html.is_empty() && !h.is_empty() {
                html = h;
            }
        }
    }

    (text, html)
}

// --- IMAP connection ---

fn connect_imap(
    server: &str,
    port: u16,
    user: &str,
    password: &str,
) -> Result<Session<TlsStream<std::net::TcpStream>>, String> {
    let tls = native_tls::TlsConnector::builder()
        .build()
        .map_err(|e| format!("TLS error: {}", e))?;

    let client = imap::connect((server, port), server, &tls)
        .map_err(|e| format!("Connection error: {}", e))?;

    let session = client
        .login(user, password)
        .map_err(|e| format!("Login error: {}", e.0))?;

    Ok(session)
}

fn get_existing_uids(
    session: &mut Session<TlsStream<std::net::TcpStream>>,
    folder: &str,
) -> Result<HashSet<u32>, String> {
    session
        .select(folder)
        .map_err(|e| format!("Select error: {}", e))?;

    let mut uids = HashSet::new();
    let search = session
        .uid_search("ALL")
        .map_err(|e| format!("Search error: {}", e))?;
    for uid in search.iter() {
        uids.insert(*uid);
    }
    Ok(uids)
}

fn fetch_and_process_new_mails(
    session: &mut Session<TlsStream<std::net::TcpStream>>,
    known_uids: &mut HashSet<u32>,
    subject_re: &Option<Regex>,
    body_re: &Option<Regex>,
) -> Result<(), String> {
    let search = session
        .uid_search("UNSEEN")
        .map_err(|e| format!("Search error: {}", e))?;

    let new_uids: Vec<u32> = search
        .iter()
        .filter(|uid| !known_uids.contains(uid))
        .copied()
        .collect();

    if new_uids.is_empty() {
        return Ok(());
    }

    let uid_list: String = new_uids
        .iter()
        .map(|u| u.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let fetches = session
        .uid_fetch(&uid_list, "BODY.PEEK[]")
        .map_err(|e| format!("Fetch error: {}", e))?;

    for fetch in fetches.iter() {
        if let Some(body) = fetch.body() {
            if let Some(mail_data) = extract_mail_data(body) {
                let subject = mail_data["subject"].as_str().unwrap_or("");
                let body_text = mail_data["body_text"].as_str().unwrap_or("");
                let body_html = mail_data["body_html"].as_str().unwrap_or("");

                if matches_filters(subject, body_text, body_html, subject_re, body_re) {
                    send_event("new_mail", mail_data);
                }
            }
        }
        known_uids.insert(fetch.uid.unwrap_or(0));
    }

    Ok(())
}

// --- Watch loop ---

fn run_imap_watcher(
    cli: &Cli,
    password: &str,
    subject_re: &Option<Regex>,
    body_re: &Option<Regex>,
) {
    let mut backoff = 1u64;

    loop {
        eprintln!("Connecting to {}:{}...", cli.server, cli.port);

        let mut session = match connect_imap(&cli.server, cli.port, &cli.user, password) {
            Ok(s) => {
                backoff = 1;
                s
            }
            Err(e) => {
                eprintln!("Connection failed: {}. Retrying in {}s...", e, backoff);
                std::thread::sleep(Duration::from_secs(backoff));
                backoff = (backoff * 2).min(300);
                continue;
            }
        };

        // Get existing UIDs (don't fire events for these)
        let mut known_uids = match get_existing_uids(&mut session, &cli.folder) {
            Ok(uids) => {
                eprintln!(
                    "Connected. Monitoring {} ({} existing messages)",
                    cli.folder,
                    uids.len()
                );
                uids
            }
            Err(e) => {
                eprintln!("Failed to get existing UIDs: {}", e);
                let _ = session.logout();
                std::thread::sleep(Duration::from_secs(backoff));
                backoff = (backoff * 2).min(300);
                continue;
            }
        };

        // Check if server supports IDLE
        let has_idle = session
            .capabilities()
            .map(|caps| caps.has_str("IDLE"))
            .unwrap_or(false);

        if has_idle {
            eprintln!("Using IDLE for real-time notifications");
        } else {
            eprintln!("IDLE not supported, polling every {}s", cli.poll_interval);
        }

        // Main watch loop
        loop {
            if has_idle {
                // Use IDLE - wait for server notification
                let idle_handle = match session.idle() {
                    Ok(h) => h,
                    Err(e) => {
                        eprintln!("IDLE setup failed: {}. Reconnecting...", e);
                        break;
                    }
                };

                // Wait for IDLE response (timeout after poll_interval to periodically check)
                // Handle is dropped after wait, returning control to session
                if let Err(e) =
                    idle_handle.wait_with_timeout(Duration::from_secs(cli.poll_interval))
                {
                    eprintln!("IDLE wait failed: {}. Reconnecting...", e);
                    break;
                }
            } else {
                std::thread::sleep(Duration::from_secs(cli.poll_interval));

                // Re-select folder to refresh
                if let Err(e) = session.select(&cli.folder) {
                    eprintln!("Select failed: {}. Reconnecting...", e);
                    break;
                }
            }

            // Check for new mail
            if let Err(e) =
                fetch_and_process_new_mails(&mut session, &mut known_uids, subject_re, body_re)
            {
                eprintln!("Fetch failed: {}. Reconnecting...", e);
                break;
            }
        }

        let _ = session.logout();
        eprintln!("Reconnecting in {}s...", backoff);
        std::thread::sleep(Duration::from_secs(backoff));
        backoff = (backoff * 2).min(300);
    }
}

// --- Main ---

fn resolve_password(cli: &Cli) -> String {
    if let Some(ref env_var) = cli.password_env {
        std::env::var(env_var).unwrap_or_else(|_| {
            eprintln!("Environment variable '{}' not set", env_var);
            std::process::exit(1);
        })
    } else {
        cli.password.clone().unwrap_or_else(|| {
            eprintln!("Either --password or --password-env is required");
            std::process::exit(1);
        })
    }
}

fn main() {
    let cli = Cli::parse();

    let subject_re = compile_filter(&cli.subject_match);
    let body_re = compile_filter(&cli.body_match);

    // Read JSON-RPC messages from stdin
    let stdin = io::stdin();
    let reader = stdin.lock();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let msg: JsonRpcMessage = match serde_json::from_str(trimmed) {
            Ok(m) => m,
            Err(_) => continue,
        };

        match msg.method.as_str() {
            "initialize" => {
                if let Some(id) = msg.id {
                    send_response(
                        id,
                        serde_json::json!({
                            "name": "imap-watcher",
                            "version": "0.1.0",
                            "description": "Watches an IMAP mailbox for new emails (built-in)",
                            "server": cli.server,
                            "folder": cli.folder,
                        }),
                    );
                }

                // Start watching in a separate thread
                let thread_cli = cli.clone();
                let thread_password = resolve_password(&cli);
                let subject_re2 = subject_re.clone();
                let body_re2 = body_re.clone();

                std::thread::spawn(move || {
                    run_imap_watcher(&thread_cli, &thread_password, &subject_re2, &body_re2);
                });
            }
            "shutdown" => {
                if let Some(id) = msg.id {
                    send_response(id, serde_json::json!({"status": "ok"}));
                }
                std::process::exit(0);
            }
            _ => {}
        }
    }

    // stdin closed
    std::process::exit(0);
}
