# clawser-fetch

Antidetect HTTP client powered by Chromium's real network stack. Chrome-identical TLS (JA3/JA4), HTTP/2 fingerprints, header ordering, and cookie behavior.

## Quick Start

```rust
use clawser_fetch::{Session, Seed};

// Random identity — unique TLS + HTTP/2 fingerprint
let (session, seed) = Session::random()?;
println!("Seed: {}", seed.to_hex()); // save this 80-char hex string

// Make requests — cookies persist, redirects followed
let resp = session.get("https://example.com")?.send()?;
println!("{}", resp.status());
println!("{}", resp.text()?);

// Replay same identity later
let seed = Seed::from_hex("abc123...").unwrap();
let session = Session::from_seed(&seed)?;
```

## What's spoofed

| Layer | Feature | Status |
|-------|---------|--------|
| TLS | Cipher suites, extensions, GREASE (JA3/JA4) | Real Chrome (BoringSSL) |
| HTTP/2 | SETTINGS frame, pseudo-header order, WINDOW_UPDATE, GREASE | Randomized per-session |
| Headers | User-Agent, Accept-Language, sec-ch-ua, sec-fetch-* | From seed profile |
| Cookies | Full cookie jar with redirect chain handling | Chromium CookieMonster |
| Connection | Socket pooling (32/6), keep-alive, preconnect | Chrome-identical |

## Native Library

This crate requires the `clawser_fetch` native library. Set `CLAWSER_LIB_DIR` to the directory containing the prebuilt binary, or place it in `clawser-sys/native/`.

Prebuilt binaries: [GitHub Releases](https://github.com/kaykay0201/clawser-fetch-engine/releases)

## License

MIT OR Apache-2.0
