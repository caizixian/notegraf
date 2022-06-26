mod common;

use common::*;
use reqwest::Client;

use notegraf::NoteLocator;
use serde_json::{json, Value};

#[tokio::test]
async fn new_note() {
    let app = spawn_app().await;
    let client = Client::new();

    let response = client
        // Use the returned application address
        .post(&format!("{}/api/v1/note", &app.address))
        .json(&json!({
            "title": "My title",
            "note_inner": "# Hey Markdown Note\n## H2"
        }))
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");
    println!("{}", response);
    assert!(response.is_object());
    let loc = response.as_object().unwrap();
    assert!(loc.contains_key("Specific"));
}

async fn create_note_helper(
    client: &Client,
    address: &str,
    title: &str,
    note_inner: &str,
) -> NoteLocator {
    client
        .post(&format!("{}/api/v1/note", address))
        .json(&json!({
            "title": title.to_owned(),
            "note_inner": note_inner.to_owned()
        }))
        .send()
        .await
        .expect("Failed to execute request.")
        .json()
        .await
        .expect("Failed to parse response")
}

#[tokio::test]
async fn note_retrive() {
    let app = spawn_app().await;
    let client = Client::new();

    let loc1 = create_note_helper(&client, &app.address, "title", "## body text").await;
    let response = client
        .get(&format!(
            "{}/api/v1/note/{}",
            &app.address,
            loc1.get_id().as_ref()
        ))
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");

    assert!(response.is_object());
    assert_eq!(response["id"], loc1.get_id().as_ref());
    assert_eq!(response["revision"], loc1.get_revision().unwrap().as_ref());
    assert_eq!(response["next"], Value::Null);
    assert_eq!(response["title"], "title");
    assert_eq!(response["note_inner"], "## body text");
}
