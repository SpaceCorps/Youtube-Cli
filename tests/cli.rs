//! Integration tests for youtube CLI using an in-process TCP mock HTTP server.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct Recorded {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Option<Value>,
}

type Route = (&'static str, &'static str, u16, Value);

struct Mock {
    url: String,
    log: Arc<Mutex<Vec<Recorded>>>,
}

impl Mock {
    fn start(routes: Vec<Route>) -> Mock {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/v2/", listener.local_addr().unwrap());
        let log = Arc::new(Mutex::new(Vec::new()));
        let log2 = log.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let routes = routes.clone();
                let log = log2.clone();
                std::thread::spawn(move || {
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 {
                        return;
                    }
                    let mut parts = line.split_whitespace();
                    let method = parts.next().unwrap_or("").to_string();
                    let path = parts.next().unwrap_or("").trim_start_matches("/v2/").to_string();
                    let mut headers = Vec::new();
                    let mut len = 0usize;
                    loop {
                        let mut h = String::new();
                        reader.read_line(&mut h).unwrap();
                        let h = h.trim_end();
                        if h.is_empty() {
                            break;
                        }
                        if let Some((k, v)) = h.split_once(':') {
                            let (k, v) = (k.trim().to_lowercase(), v.trim().to_string());
                            if k == "content-length" {
                                len = v.parse().unwrap_or(0);
                            }
                            headers.push((k, v));
                        }
                    }
                    let mut buf = vec![0; len];
                    reader.read_exact(&mut buf).unwrap();
                    let body = (len > 0).then(|| serde_json::from_slice(&buf).unwrap());
                    log.lock().unwrap().push(Recorded { method: method.clone(), path: path.clone(), headers, body });

                    let (status, resp) = routes
                        .iter()
                        .find(|(m, p, _, _)| *m == method && *p == path)
                        .map(|(_, _, s, b)| (*s, b.clone()))
                        .unwrap_or((404, json!({"error": {"message": "not found"}})));
                    let text = resp.to_string();
                    let _ = write!(
                        stream,
                        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
                        text.len()
                    );
                });
            }
        });
        Mock { url, log }
    }

    fn requests(&self) -> Vec<Recorded> {
        self.log.lock().unwrap().clone()
    }

    fn last(&self, method: &str) -> Recorded {
        self.requests().into_iter().rev().find(|r| r.method == method).expect("no such request")
    }
}

struct Env {
    dir: PathBuf,
    api: String,
}

impl Env {
    fn new(mock: &Mock) -> Env {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "youtube-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Env { dir, api: mock.url.clone() }
    }

    fn run(&self, args: &[&str]) -> Output {
        self.run_with_env(args, &[])
    }

    fn run_with_env(&self, args: &[&str], extra_env: &[(&str, &str)]) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_youtube"));
        cmd.args(args)
            .env("YOUTUBE_CONFIG_DIR", &self.dir)
            .env("YOUTUBE_SECRET_STORE", "plaintext")
            .env("YOUTUBE_ALLOW_PLAINTEXT_STORE", "1")
            .env("YOUTUBE_API_URL", &self.api)
            .env_remove("APIFY_TOKEN")
            .env_remove("YOUTUBE_API_KEY");
        for (k, v) in extra_env {
            cmd.env(k, v);
        }
        cmd.output().unwrap()
    }

    fn json(&self, args: &[&str]) -> (i32, Value, Value) {
        let mut all = args.to_vec();
        all.push("--json");
        let out = self.run(&all);
        let parse = |b: &[u8]| {
            let s = String::from_utf8_lossy(b);
            let s = s.lines().filter(|l| !l.starts_with("warning:")).collect::<Vec<_>>().join("\n");
            serde_json::from_str(&s).unwrap_or(Value::Null)
        };
        (out.status.code().unwrap(), parse(&out.stdout), parse(&out.stderr))
    }

    fn with_account(self, name: &str, key: &str) -> Env {
        let (code, out, err) = self.json(&["accounts", "add", name, "--api-key", key]);
        assert_eq!(code, 0, "{err}");
        assert_eq!(out["status"], "added");
        self
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn me() -> Route {
    (
        "GET",
        "users/me",
        200,
        json!({
            "data": {
                "id": "u1",
                "username": "janedoe",
                "email": "jane@example.com",
                "plan": {"name": "FREE"}
            }
        }),
    )
}

fn scraper_route() -> Route {
    (
        "POST",
        "acts/streamers~youtube-scraper/run-sync-get-dataset-items",
        200,
        json!([
            {
                "id": "dQw4w9WgXcQ",
                "title": "Never Gonna Give You Up",
                "url": "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
                "channelName": "Rick Astley",
                "viewCount": 1500000000i64
            }
        ]),
    )
}

#[test]
fn accounts_lifecycle() {
    let mock = Mock::start(vec![me()]);
    let env = Env::new(&mock).with_account("work", "apify_key_1");

    assert_eq!(mock.last("GET").headers.iter().find(|(k, _)| k == "authorization").unwrap().1, "Bearer apify_key_1");

    let (code, out, _) = env.json(&["accounts", "list"]);
    assert_eq!(code, 0);
    assert_eq!(out["count"], 1);
    assert_eq!(out["accounts"][0]["identity"], "janedoe");
    assert_eq!(out["accounts"][0]["organization"], "FREE");
    assert_eq!(out["accounts"][0]["keyStatus"], "stored");
    assert_eq!(out["secretStore"], "plaintext");

    let (_, out, _) = env.json(&["accounts", "list", "--check"]);
    assert_eq!(out["accounts"][0]["keyStatus"], "valid");

    // Case-insensitive duplicate needs --force
    let (code, _, err) = env.json(&["accounts", "add", "WORK", "--api-key", "x"]);
    assert_eq!(code, 6);
    assert!(err["remediation"].as_str().unwrap().contains("--force"));

    let (code, out, _) = env.json(&["accounts", "test", "Work"]);
    assert_eq!(code, 0);
    assert_eq!(out["keyStatus"], "valid");
    assert_eq!(out["identity"], "janedoe");

    // Refuse remove without terminal or --yes
    let (code, _, err) = env.json(&["accounts", "remove", "work"]);
    assert_eq!(code, 6);
    assert_eq!(err["remediation"], "youtube accounts remove work --yes");

    let (code, out, _) = env.json(&["accounts", "remove", "work", "--yes"]);
    assert_eq!(code, 0);
    assert_eq!(out["status"], "removed");

    let (_, out, _) = env.json(&["accounts", "list"]);
    assert_eq!(out["count"], 0);
}

#[test]
fn search_command() {
    let mock = Mock::start(vec![me(), scraper_route()]);
    let env = Env::new(&mock).with_account("work", "apify_key_1");

    let (code, out, _) = env.json(&[
        "search",
        "Rick Astley",
        "--account",
        "work",
        "--max-results",
        "5",
        "--hd",
        "--four-k",
        "--download-subs",
        "--subs-lang",
        "en",
    ]);
    assert_eq!(code, 0);
    assert_eq!(out[0]["title"], "Never Gonna Give You Up");

    let last_post = mock.last("POST");
    assert_eq!(last_post.headers.iter().find(|(k, _)| k == "authorization").unwrap().1, "Bearer apify_key_1");

    let body = last_post.body.unwrap();
    assert_eq!(body["searchQueries"][0], "Rick Astley");
    assert_eq!(body["maxResults"], 5);
    assert_eq!(body["isHD"], true);
    assert_eq!(body["is4K"], true);
    assert_eq!(body["downloadSubtitles"], true);
    assert_eq!(body["subtitlesLanguage"], "en");
}

#[test]
fn search_direct_api_key() {
    let mock = Mock::start(vec![scraper_route()]);
    let env = Env::new(&mock);

    let (code, out, _) = env.json(&["search", "rust tutorial", "--api-key", "direct_token_123"]);
    assert_eq!(code, 0);
    assert_eq!(out[0]["title"], "Never Gonna Give You Up");

    let last_post = mock.last("POST");
    assert_eq!(last_post.headers.iter().find(|(k, _)| k == "authorization").unwrap().1, "Bearer direct_token_123");
}

#[test]
fn scrape_command() {
    let mock = Mock::start(vec![me(), scraper_route()]);
    let env = Env::new(&mock).with_account("work", "apify_key_1");

    let (code, out, _) = env.json(&[
        "scrape",
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
        "-a",
        "work",
        "--sort",
        "POPULAR",
        "--since",
        "7 days",
    ]);
    assert_eq!(code, 0);
    assert_eq!(out[0]["id"], "dQw4w9WgXcQ");

    let last_post = mock.last("POST");
    let body = last_post.body.unwrap();
    assert_eq!(body["startUrls"][0]["url"], "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    assert_eq!(body["sortVideosBy"], "POPULAR");
    assert_eq!(body["oldestPostDate"], "7 days");
}

#[test]
fn apify_token_env_fallback() {
    let mock = Mock::start(vec![scraper_route()]);
    let env = Env::new(&mock);

    let out = env.run_with_env(&["search", "coding", "--json"], &[("APIFY_TOKEN", "env_token_456")]);
    assert_eq!(out.status.code().unwrap(), 0);

    let last_post = mock.last("POST");
    assert_eq!(last_post.headers.iter().find(|(k, _)| k == "authorization").unwrap().1, "Bearer env_token_456");
}

#[test]
fn auth_required_when_no_credentials() {
    let mock = Mock::start(vec![]);
    let env = Env::new(&mock);

    let (code, _, err) = env.json(&["search", "query"]);
    assert_eq!(code, 7); // NoAccount
    assert_eq!(err["code"], "no_account");
    assert!(err["remediation"].as_str().unwrap().contains("youtube login"));
}

#[test]
fn error_mapping_rate_limited() {
    let mock = Mock::start(vec![(
        "POST",
        "acts/streamers~youtube-scraper/run-sync-get-dataset-items",
        429,
        json!({"error": {"message": "Rate limit exceeded"}}),
    )]);
    let env = Env::new(&mock);

    let (code, _, err) = env.json(&["search", "query", "--api-key", "key"]);
    assert_eq!(code, 5); // RateLimited
    assert_eq!(err["code"], "rate_limited");
}

#[test]
fn agent_readme() {
    let mock = Mock::start(vec![]);
    let env = Env::new(&mock);

    let out = env.run(&["agent-readme"]);
    assert_eq!(out.status.code().unwrap(), 0);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("# youtube - agent operating manual"));

    let (code, out, _) = env.json(&["agent-readme"]);
    assert_eq!(code, 0);
    assert_eq!(out["tool"], "youtube");
    assert!(out["rules"].is_array());
    assert_eq!(out["exitCodes"]["0"], "ok");
    assert_eq!(out["exitCodes"]["3"], "auth_required - stop, surface the remediation to a human");
}

#[test]
fn yaml_default_output() {
    let mock = Mock::start(vec![scraper_route()]);
    let env = Env::new(&mock);

    let out = env.run(&["search", "test", "--api-key", "key"]);
    assert_eq!(out.status.code().unwrap(), 0);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("title: Never Gonna Give You Up"));
    assert!(stdout.contains("channelName: Rick Astley"));
}
