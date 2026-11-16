//! The sender against a local server on this machine's loopback: what it
//! sends, and how each status reads.

use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::net::TcpListener;
use std::sync::mpsc;
use std::time::Duration;

use reports::send::{self, derived_id, next_wait, post, Attempt, Destination};

/// What the server saw.
struct Seen {
    request_line: String,
    headers: Vec<(String, String)>,
    body: String,
}

/// A server that answers one request with `status`, and tells what it saw.
fn server(status: u16) -> (String, mpsc::Receiver<Seen>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/v1/reports", listener.local_addr().unwrap());
    let (tell, seen) = mpsc::channel();
    std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut request_line = String::new();
        reader.read_line(&mut request_line).unwrap();
        let mut headers = Vec::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let line = line.trim_end();
            if line.is_empty() {
                break;
            }
            let (name, value) = line.split_once(':').unwrap();
            headers.push((name.to_ascii_lowercase(), value.trim().to_string()));
        }
        let length: usize = headers
            .iter()
            .find(|(n, _)| n == "content-length")
            .map_or(0, |(_, v)| v.parse().unwrap());
        let mut body = vec![0; length];
        reader.read_exact(&mut body).unwrap();
        let mut stream = stream;
        write!(
            stream,
            "HTTP/1.1 {status} X\r\ncontent-length: 0\r\nconnection: close\r\n\r\n"
        )
        .unwrap();
        let _ = tell.send(Seen {
            request_line: request_line.trim_end().to_string(),
            headers,
            body: String::from_utf8(body).unwrap(),
        });
    });
    (url, seen)
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

const BODY: &str = "{\n  \"format\": 1,\n  \"kind\": \"panic\"\n}";
const ID: &str = "a1b2";

#[test]
fn a_report_is_posted_as_its_exact_bytes_with_its_id_and_key() {
    let (url, seen) = server(201);
    let to = Destination::new(url, Some("test-key".into())).unwrap();
    assert_eq!(
        runtime().block_on(post(&to, BODY.into(), ID.into())),
        Attempt::Sent
    );
    let seen = seen.recv_timeout(Duration::from_secs(10)).unwrap();
    assert_eq!(seen.request_line, "POST /v1/reports HTTP/1.1");
    assert_eq!(seen.body, BODY);
    let header = |name: &str| {
        seen.headers
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.clone())
    };
    assert_eq!(header("content-type").as_deref(), Some("application/json"));
    assert_eq!(header("authorization").as_deref(), Some("Bearer test-key"));
    assert_eq!(header("idempotency-key").as_deref(), Some(ID));
}

#[test]
fn a_4xx_is_refused_and_a_5xx_or_no_server_is_worth_retrying() {
    let (url, _seen) = server(400);
    let to = Destination::new(url, None).unwrap();
    assert_eq!(
        runtime().block_on(post(&to, BODY.into(), ID.into())),
        Attempt::Refused(400)
    );
    let (url, _seen) = server(503);
    let to = Destination::new(url, None).unwrap();
    assert!(matches!(
        runtime().block_on(post(&to, BODY.into(), ID.into())),
        Attempt::Failed(_)
    ));
    // Nothing listens on this port any more.
    let gone = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap();
    let to = Destination::new(format!("http://{gone}/"), None).unwrap();
    assert!(matches!(
        runtime().block_on(post(&to, BODY.into(), ID.into())),
        Attempt::Failed(_)
    ));
    // Too large: refused before anything is sent.
    let big = "x".repeat(send::MAX_BYTES + 1);
    assert_eq!(
        runtime().block_on(post(&to, big, ID.into())),
        Attempt::Refused(413)
    );
}

#[test]
fn only_https_or_this_macs_loopback_is_a_destination() {
    assert!(Destination::new("https://reports.example.test/v1".into(), None).is_some());
    assert!(Destination::new("http://127.0.0.1:8080/".into(), None).is_some());
    assert!(Destination::new("http://localhost/x".into(), None).is_some());
    assert!(Destination::new("http://reports.example.test/".into(), None).is_none());
    assert!(Destination::new("ftp://x/".into(), None).is_none());
    assert!(Destination::new("https://".into(), None).is_none());
    let to = Destination::new("https://reports.example.test:443/v1?x".into(), None).unwrap();
    assert_eq!(to.host(), "reports.example.test:443");
}

#[test]
fn an_older_reports_id_is_stable_and_differs_between_reports() {
    let one = derived_id("capsule-1", "0000001700000000000-panic.json");
    assert_eq!(one, derived_id("capsule-1", "0000001700000000000-panic"));
    assert!(isonomic_reports::id::is_id(&one), "{one}");
    assert!(!one.contains("capsule-1"));
    assert_ne!(
        one,
        derived_id("capsule-1", "0000001700000000001-panic.json")
    );
    assert_ne!(
        one,
        derived_id("capsule-2", "0000001700000000000-panic.json")
    );
}

#[test]
fn retries_double_to_an_hour_and_stop_after_a_day() {
    let minute = Duration::from_secs(60);
    assert_eq!(next_wait(1, Duration::ZERO), Some(send::FIRST_WAIT));
    assert_eq!(next_wait(2, minute), Some(minute));
    assert_eq!(next_wait(3, minute), Some(2 * minute));
    assert_eq!(next_wait(20, minute), Some(send::LONGEST_WAIT));
    assert_eq!(next_wait(30, send::GIVE_UP_AFTER - minute), None);
    assert_eq!(next_wait(1, send::GIVE_UP_AFTER), None);
}
