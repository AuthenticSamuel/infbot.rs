use std::env;

use posthog_rs::{Client, Event};

pub async fn init_client() -> Option<Client> {
    let (Some(key), Some(host)) = (
        env::var("POSTHOG_PROJECT_KEY").ok(),
        env::var("POSTHOG_PROJECT_HOST").ok(),
    ) else {
        return None;
    };

    let options = posthog_rs::ClientOptionsBuilder::default()
        .api_key(key)
        .api_endpoint(host)
        .build()
        .ok()?;

    return Some(posthog_rs::client(options).await);
}

pub async fn capture_event(client: &Client, event_name: &str, distinct_id: &str) {
    let event = Event::new(event_name, distinct_id);

    if let Err(err) = client.capture(event).await {
        eprintln!("PostHog capture failed: {err}");
    };
}

pub async fn capture_event_with_props<I, K>(
    client: &Client,
    event_name: &str,
    distinct_id: &str,
    properties: I,
) where
    I: IntoIterator<Item = (K, serde_json::Value)>,
    K: Into<String>,
{
    let mut event = Event::new(event_name, distinct_id);

    for (k, v) in properties {
        let _ = event.insert_prop(k, v);
    }

    if let Err(err) = client.capture(event).await {
        eprintln!("PostHog capture failed: {err}");
    };
}
