# clawser-fetch

Antidetect HTTP client powered by Chromium's real network stack. Chrome-identical TLS (JA3/JA4), HTTP/2 fingerprints, header ordering, and cookie behavior.

## Install

```bash
cargo add clawser-fetch
```

## Quick Start

```rust
use std::sync::Arc;
use clawser_fetch::{FetchEngine, Seed};

// Create engine once at app startup
let engine = Arc::new(FetchEngine::new());

// New random identity — unique TLS + HTTP/2 fingerprint + cookie jar
let (seed, resp) = engine.fetch_random("GET", "https://example.com")?;
println!("Status: {}", resp.status());
println!("Seed: {}", seed.to_hex()); // save this 80-char hex string

// Same identity — cookies persist automatically
let resp = engine.fetch(&seed, "GET", "https://example.com/page2")?;

// Replay identity later (same fingerprint, fresh cookies)
let seed = Seed::from_hex("abc123...").unwrap();
let resp = engine.fetch(&seed, "GET", "https://example.com")?;
```

## Custom Headers & POST

```rust
// Random identity WITH custom headers (new in 0.2.0)
let (seed, resp) = engine.request_random("POST", "https://api.example.com/data")?
    .header("Authorization", "Bearer my-token")
    .header("X-Custom", "value")
    .json(r#"{"key": "value"}"#)
    .timeout_ms(5000)
    .send_with_seed()?;

println!("Seed: {} Status: {}", seed.to_hex(), resp.status());

// Full control via request builder (existing seed)
let resp = engine.request(&seed, "POST", "https://api.example.com/data")?
    .header("Authorization", "Bearer my-token")
    .json(r#"{"key": "value"}"#)
    .timeout_ms(5000)
    .send()?;

println!("{}", resp.text()?);

// Override default Chrome headers
let resp = engine.request(&seed, "GET", "https://example.com")?
    .header("Accept-Language", "fr-FR,fr;q=0.9")
    .header("sec-ch-ua-platform", "\"Linux\"")
    .send()?;

// Two-step: create identity first, request later
let seed = engine.create_random()?;
let resp = engine.request(&seed, "GET", "https://example.com")?
    .header("Referer", "https://google.com")
    .send()?;
```

## Cookie Management

```rust
// Cookies accumulate across requests per identity
engine.fetch(&seed, "GET", "https://site.com/login")?;   // receives Set-Cookie
engine.fetch(&seed, "GET", "https://site.com/dashboard")?; // sends cookies automatically

// Read all cookies for an identity
let cookies_json = engine.cookies(&seed)?;
println!("{}", cookies_json);
// [{"name":"session","value":"abc","domain":".site.com","path":"/","secure":true,"httponly":false}]
```

## Multi-threaded

```rust
use std::sync::Arc;
use std::thread;

let engine = Arc::new(FetchEngine::new());

let handles: Vec<_> = (0..10).map(|_| {
    let e = engine.clone();
    thread::spawn(move || {
        let (seed, resp) = e.fetch_random("GET", "https://httpbin.org/get").unwrap();
        println!("seed={} status={}", seed.to_hex(), resp.status());
    })
}).collect();

for h in handles {
    h.join().unwrap();
}

println!("Active sessions: {}", engine.session_count());
```

## Session Lifecycle

```rust
// List all active identities
let seeds = engine.seeds()?;

// Drop a specific identity (frees memory + cookies)
engine.drop_session(&seed)?;

// Check active count
println!("{} sessions alive", engine.session_count());
```

## What's spoofed per seed

| Layer | Feature | Controlled by |
|-------|---------|---------------|
| TLS | Cipher suite order (JA3) | `canvas_seed` |
| TLS | Signature algorithm order (JA4) | `webgl_seed` |
| HTTP/2 | SETTINGS frame profile | `audio_seed` |
| HTTP/2 | Pseudo-header order | `canvas_seed` |
| HTTP/2 | WINDOW_UPDATE delta | `client_rects_seed` |
| Headers | User-Agent, sec-ch-ua, Accept-Language | `hw_seed` |
| Hardware | GPU, CPU cores, screen resolution | `hw_seed` |
| Cookies | Full cookie jar per session | Automatic |
| Redirects | Follow chain with correct cookie forwarding | Automatic |

Same seed = identical fingerprint. Different seed = different identity.

## Native Library

This crate requires the `clawser_fetch` native library (Chromium net stack). It auto-downloads from [GitHub Releases](https://github.com/kaykay0201/clawser-fetch-engine/releases) during `cargo build`.

Override with `CLAWSER_LIB_DIR` env var if needed.

## License

MIT OR Apache-2.0
