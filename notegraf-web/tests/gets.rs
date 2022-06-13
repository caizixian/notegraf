mod common;
use common::*;

use serde_json::Value;

#[tokio::test]
async fn check_get_note() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        // Use the returned application address
        .get(&format!("{}/api/v1/note/note-1", &app.address))
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");

    assert!(matches!(response, Value::Object(_)));
    assert_eq!(response["id"], "note-1");
    assert_eq!(response["revision"], "revision-1");
    assert_eq!(response["next"], "note-2");

    let response = client
        .get(&format!(
            "{}/api/v1/note/note-0/revision/revision-0",
            &app.address
        ))
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");

    assert!(matches!(response, Value::Object(_)));
    assert_eq!(response["id"], "note-0");
    assert_eq!(response["revision"], "revision-0");
    assert_eq!(response["next"], Value::Null);
}
