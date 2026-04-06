use clawser_fetch::Session;

fn main() {
    // Create a session with an antidetect profile.
    // Pass the path to your clawser profile JSON.
    let config = std::env::args().nth(1).unwrap_or_default();
    let session = if config.is_empty() {
        Session::default_session().expect("Failed to create session")
    } else {
        Session::new(&config).expect("Failed to create session with config")
    };

    // Simple GET request — follows redirects, sends cookies.
    let resp = session
        .get("https://httpbin.org/get")
        .expect("Failed to create request")
        .send()
        .expect("Request failed");

    println!("Status: {}", resp.status());
    println!("URL: {}", resp.url());
    println!("Headers: {:#?}", resp.headers());
    println!("Body:\n{}", resp.text().unwrap_or_else(|_| "[binary]".into()));
}
