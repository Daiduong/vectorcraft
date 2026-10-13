//! The backend the server uses with no mode flag: the app when one answers, an in-process
//! headless session while none does — including an app that starts after the session has (#1470).

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::{Auto, Backend, Server};

/// How long a test lets `Auto` wait before looking for the app again.
const PROBE: Duration = Duration::from_millis(10);

/// An `Auto` that looks for the app again almost at once.
fn auto(addr: impl Into<String>) -> Auto {
    let mut a = Auto::new(addr);
    a.re_probe = PROBE;
    a
}

/// A port with nothing listening: connecting to it is refused, the way an app that isn't running
/// looks. The socket is held rather than dropped so nothing else takes the port meanwhile.
fn hold_port() -> (u16, socket2::Socket) {
    let held = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::STREAM, None).unwrap();
    held.bind(&std::net::SocketAddr::from(([127, 0, 0, 1], 0)).into()).unwrap();
    let port = held.local_addr().unwrap().as_socket().unwrap().port();
    (port, held)
}

/// Take the held port over: what starting the app does. The window between the two is a
/// statement long; a test that loses the port to another program then has already lost.
fn take_over(held: socket2::Socket, port: u16) -> TcpListener {
    drop(held);
    TcpListener::bind(("127.0.0.1", port)).unwrap_or_else(|e| panic!("port {port} was taken while the app was starting: {e}"))
}

/// A stand-in for the desktop app: every request is answered with a marker result and the
/// request's `id`, the way the real control server echoes it. `close_after` closes a connection
/// once that many replies have been written on it (an app that goes away); `None` waits for the
/// client to close it. The thread ends after `connections` served connections, dropping its
/// listener, so a later connect is refused.
fn spawn_app(listener: TcpListener, connections: usize, close_after: Option<usize>) -> std::thread::JoinHandle<Vec<Value>> {
    std::thread::spawn(move || {
        let mut seen = Vec::new();
        let mut served = 0usize;
        for stream in listener.incoming().flatten() {
            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
            stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
            let mut out = stream.try_clone().unwrap();
            let mut reader = BufReader::new(stream);
            let mut replies = 0usize;
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 {
                    break;
                }
                let Ok(msg) = serde_json::from_str::<Value>(&line) else { break };
                seen.push(msg.clone());
                let reply = json!({"ok": true, "result": {"from": "app"}, "id": msg.get("id").cloned().unwrap_or(Value::Null)});
                if writeln!(out, "{reply}").and_then(|()| out.flush()).is_err() {
                    break;
                }
                replies += 1;
                if close_after == Some(replies) {
                    // Hold the connection a moment, so the client's next request meets the close
                    // rather than a write into a socket that is still open.
                    std::thread::sleep(20 * PROBE);
                    break;
                }
            }
            served += 1;
            if served == connections {
                break;
            }
        }
        seen
    })
}

/// One request through the protocol, exactly as a client sends it.
fn rpc(s: &mut Server, id: u64, method: &str, params: Value) -> Value {
    let line = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}).to_string();
    let reply = s.handle_line(&line).expect("a request gets a reply");
    serde_json::from_str(&reply).expect("the reply is JSON")
}

/// Run one tool and return its result object.
fn tool(s: &mut Server, id: u64, name: &str) -> Value {
    rpc(s, id, "tools/call", json!({"name": name, "arguments": {}}))["result"].clone()
}

/// The log records the server sent as `notifications/message` since the last drain.
fn notes(s: &mut Server) -> Vec<Value> {
    s.take_notifications().iter().filter_map(|l| serde_json::from_str(l).ok()).collect()
}

#[test]
fn an_app_that_is_running_answers() {
    let _turn = crate::logging::test_turn();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let app = spawn_app(listener, 1, None);
    let mut a = auto(addr.clone());
    let r = a.call("document.inspect", json!({})).unwrap();
    assert_eq!(r["from"], "app", "{r}");
    assert!(a.on_app());
    assert!(a.has_ui());
    assert_eq!(a.describe(), format!("remote {addr}"));
    drop(a);
    app.join().unwrap();
}

#[test]
fn without_an_app_the_session_is_headless_and_says_so() {
    let _turn = crate::logging::test_turn();
    let (port, held) = hold_port();
    let mut a = auto(format!("127.0.0.1:{port}"));
    let r = a.call("document.inspect", json!({})).unwrap();
    assert!(r.get("from").is_none(), "{r}");
    assert!(!a.on_app(), "nothing is listening");
    assert!(!a.has_ui());
    let described = a.describe();
    assert!(described.starts_with("headless") && described.contains("takes over"), "{described}");
    // The session is a real document, so drawing works without an app.
    a.call("engine.execute", json!({"command": "shape.rectangle", "params": {"x": 1, "y": 1, "width": 5, "height": 5}})).unwrap();
    drop(held);
}

#[test]
fn an_app_started_later_is_reached() {
    let _turn = crate::logging::test_turn();
    let (port, held) = hold_port();
    let mut a = auto(format!("127.0.0.1:{port}"));
    assert!(!a.on_app(), "no app is running yet");
    let before = a.call("document.inspect", json!({})).unwrap();
    assert!(before.get("from").is_none(), "the headless session answered: {before}");
    assert!(!a.on_app());

    // The user opens VectorCraft while the session is already running.
    let listener = take_over(held, port);
    let app = spawn_app(listener, 1, None);
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut answered = None;
    while Instant::now() < deadline {
        if a.on_app() {
            answered = Some(a.call("document.inspect", json!({})).unwrap());
            break;
        }
        let _ = a.call("document.inspect", json!({}));
        std::thread::sleep(PROBE);
    }
    let r = answered.expect("the app started during the session was never reached");
    assert_eq!(r["from"], "app", "{r}");
    assert!(a.on_app());
    drop(a);
    app.join().unwrap();
}

#[test]
fn an_app_that_goes_away_leaves_the_next_call_to_the_headless_session() {
    let _turn = crate::logging::test_turn();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let app = spawn_app(listener, 1, Some(1));
    let mut a = auto(addr);
    assert_eq!(a.call("document.inspect", json!({})).unwrap()["from"], "app");
    assert!(a.on_app());
    let lost = a.call("document.inspect", json!({}));
    assert!(lost.is_err(), "the closed connection is reported: {lost:?}");
    assert!(!a.on_app(), "the app is gone");
    let r = a.call("document.inspect", json!({})).unwrap();
    assert!(r.get("from").is_none(), "the headless session answered: {r}");
    assert!(!a.has_ui());
    drop(a);
    app.join().unwrap();
}

#[test]
fn a_ui_tool_reaches_an_app_that_started_after_the_server() {
    let _turn = crate::logging::test_turn();
    let (port, held) = hold_port();
    let mut s = Server::new(Box::new(auto(format!("127.0.0.1:{port}"))));
    // No app yet: the UI-only tool says what is missing and names the fix.
    let r = tool(&mut s, 1, "inspect_ui");
    assert_eq!(r["isError"], true, "{r}");
    let text = r["content"][0]["text"].as_str().unwrap_or_default();
    assert!(text.contains("start `vectorcraft --control 7979`"), "{text}");

    // The app starts; the server, the session and the client's connection to it all stay put.
    let listener = take_over(held, port);
    let app = spawn_app(listener, 1, None);
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut last = r;
    for id in 2..300u64 {
        last = tool(&mut s, id, "inspect_ui");
        if last["isError"] == false {
            break;
        }
        assert!(Instant::now() < deadline, "the app was never reached: {last}");
        std::thread::sleep(PROBE);
    }
    assert_eq!(last["isError"], false, "{last}");
    drop(s);
    app.join().unwrap();
}

#[test]
fn a_switch_of_backend_is_reported_to_a_client_that_asked_for_logs() {
    let _turn = crate::logging::test_turn();
    crate::logging::install();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    // The app answers one request and then goes away for good, taking its listener with it.
    let app = spawn_app(listener, 1, Some(1));
    let mut s = Server::new(Box::new(auto(addr.clone())));
    assert_eq!(tool(&mut s, 1, "doc_inspect")["isError"], false);
    assert_eq!(crate::logging::level(), None, "records are silent until a client asks");
    assert!(notes(&mut s).is_empty(), "{:?}", notes(&mut s));

    rpc(&mut s, 0, "logging/setLevel", json!({"level": "info"}));
    assert_eq!(crate::logging::level(), Some("info"));
    let _ = tool(&mut s, 2, "doc_inspect");
    let mut reported = Vec::new();
    for id in 3..100u64 {
        let _ = tool(&mut s, id, "doc_inspect");
        reported = notes(&mut s);
        if !reported.is_empty() {
            break;
        }
    }
    assert!(!reported.is_empty(), "the switch was never reported to the client");
    assert!(
        reported
            .iter()
            .any(|n| n["method"] == "notifications/message"
                && n["params"]["data"].as_str().is_some_and(|d| d.contains(&addr) && d.contains("headless"))),
        "{reported:?}"
    );
    crate::logging::set_level(None).ok();
    drop(s);
    app.join().unwrap();
}
