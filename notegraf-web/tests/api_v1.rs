mod common;

use common::*;
use reqwest::{Client, StatusCode};

use notegraf::NoteLocator;
use serde_json::{json, Value};

async fn create_note_helper(
    client: &Client,
    address: &str,
    title: &str,
    note_inner: &str,
    tags: &str,
) -> NoteLocator {
    post_note_helper(client, address, "note", title, note_inner, tags).await
}

async fn post_note_helper(
    client: &Client,
    address: &str,
    endpoint: &str,
    title: &str,
    note_inner: &str,
    tags: &str,
) -> NoteLocator {
    client
        .post(&format!("{}/api/v1/{}", address, endpoint))
        .json(&json!({
            "title": title.to_owned(),
            "note_inner": note_inner.to_owned(),
            "metadata_tags": tags.to_owned(),
            "metadata_custom_metadata": "null"
        }))
        .send()
        .await
        .expect("Failed to execute request.")
        .json()
        .await
        .expect("Failed to parse response")
}

async fn get_note_helper(client: &Client, address: &str, loc: &NoteLocator) -> Value {
    client
        .get(&format!(
            "{}/api/v1/note/{}",
            address,
            loc.get_id().as_ref()
        ))
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response")
}

#[tokio::test]
async fn new_note() {
    let app = spawn_app().await;
    let client = Client::new();

    let loc = post_note_helper(
        &client,
        &app.address,
        "note",
        "My title",
        "# Hey Markdown Note\n## H2",
        "",
    )
    .await;

    assert!(matches!(loc, NoteLocator::Specific(_, _)));
}

#[tokio::test]
async fn note_retrieve() {
    let app = spawn_app().await;
    let client = Client::new();

    let loc1 = create_note_helper(&client, &app.address, "title", "## body text", "").await;
    let response = get_note_helper(&client, &app.address, &loc1).await;

    assert!(response.is_object());
    assert_eq!(response["id"], loc1.get_id().as_ref());
    assert_eq!(response["revision"], loc1.get_revision().unwrap().as_ref());
    assert_eq!(response["next"], Value::Null);
    assert_eq!(response["title"], "title");
    assert_eq!(response["note_inner"], "## body text");
    assert!(response["metadata"]["tags"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn note_delete() {
    let app = spawn_app().await;
    let client = Client::new();

    let loc1 = create_note_helper(&client, &app.address, "title", "## body text", "").await;
    get_note_helper(&client, &app.address, &loc1).await;
    client
        .delete(&format!(
            "{}/api/v1/note/{}",
            &app.address,
            loc1.get_id().as_ref()
        ))
        .send()
        .await
        .expect("Failed to execute request.");
    let response = client
        .get(&format!(
            "{}/api/v1/note/{}",
            &app.address,
            loc1.get_id().as_ref()
        ))
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn note_update() {
    let app = spawn_app().await;
    let client = Client::new();

    let loc1 = create_note_helper(&client, &app.address, "title", "## body text", "").await;
    let loc2 = post_note_helper(
        &client,
        &app.address,
        &format!("note/{}/revision", loc1.get_id()),
        "New title",
        "New body text",
        "",
    )
    .await;

    let response = get_note_helper(&client, &app.address, &loc1).await;
    assert_eq!(response["id"], loc1.get_id().as_ref());
    assert_eq!(response["revision"], loc2.get_revision().unwrap().as_ref());
    assert_eq!(response["next"], Value::Null);
    assert_eq!(response["title"], "New title");
    assert_eq!(response["note_inner"], "New body text");
}

#[tokio::test]
async fn note_revisions() {
    let app = spawn_app().await;
    let client = Client::new();

    let loc1 = create_note_helper(&client, &app.address, "title", "## body text", "").await;
    let loc2 = post_note_helper(
        &client,
        &app.address,
        &format!("note/{}/revision", loc1.get_id()),
        "New title",
        "New body text",
        "",
    )
    .await;

    let response = client
        .get(&format!(
            "{}/api/v1/note/{}/revision",
            &app.address,
            loc1.get_id().as_ref()
        ))
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");
    assert_eq!(
        response[0]["revision"],
        loc1.get_revision().unwrap().as_ref()
    );
    assert_eq!(
        response[1]["revision"],
        loc2.get_revision().unwrap().as_ref()
    );
}

#[tokio::test]
async fn recent_notes() {
    let app = spawn_app().await;
    let client = Client::new();

    let loc1 = create_note_helper(&client, &app.address, "title", "## body text", "").await;
    let loc2 = create_note_helper(&client, &app.address, "title2", "## body text", "").await;

    let response = client
        .get(&format!("{}/api/v1/note", &app.address))
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");
    assert_eq!(response.as_array().unwrap().len(), 2);
    assert_eq!(response[1]["id"], loc1.get_id().as_ref());
    // recent note comes first
    assert_eq!(response[0]["id"], loc2.get_id().as_ref());
}

#[tokio::test]
async fn search_notes() {
    let app = spawn_app().await;
    let client = Client::new();

    let loc1 = create_note_helper(&client, &app.address, "foo", "Fizz", "").await;
    let loc2 = create_note_helper(&client, &app.address, "bar", "buzz", "").await;

    let response = client
        .get(&format!("{}/api/v1/note", &app.address))
        .query(&[("query", "fizz")])
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");
    assert_eq!(response.as_array().unwrap().len(), 1);
    assert_eq!(response[0]["id"], loc1.get_id().as_ref());

    let response = client
        .get(&format!("{}/api/v1/note", &app.address))
        .query(&[("query", "Buzz")])
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");
    assert_eq!(response.as_array().unwrap().len(), 1);
    assert_eq!(response[0]["id"], loc2.get_id().as_ref());
}

#[tokio::test]
async fn backlink() {
    let app = spawn_app().await;
    let client = Client::new();

    let loc1 = create_note_helper(&client, &app.address, "foo", "Fizz", "").await;
    let loc2 = create_note_helper(
        &client,
        &app.address,
        "bar",
        &format!("[here is a link to foo](notegraf:/note/{})", loc1.get_id()),
        "",
    )
    .await;

    let response = get_note_helper(&client, &app.address, &loc1).await;
    assert_eq!(
        response["references"].as_array().unwrap()[0],
        loc2.get_id().as_ref()
    );
}

#[tokio::test]
async fn add_branch() {
    let app = spawn_app().await;
    let client = Client::new();

    let loc1 = create_note_helper(&client, &app.address, "title", "## body text", "").await;
    let loc2 = post_note_helper(
        &client,
        &app.address,
        &format!("note/{}/branch", loc1.get_id()),
        "child title",
        "New body text",
        "",
    )
    .await;

    let response = get_note_helper(&client, &app.address, &loc1).await;
    assert_eq!(
        response["branches"].as_array().unwrap()[0],
        loc2.get_id().as_ref()
    );

    let response = get_note_helper(&client, &app.address, &loc2).await;
    assert_eq!(response["parent"].as_str().unwrap(), loc1.get_id().as_ref());
}

#[tokio::test]
async fn append_note() {
    let app = spawn_app().await;
    let client = Client::new();

    let loc1 = create_note_helper(&client, &app.address, "title", "## body text", "").await;
    let loc2 = post_note_helper(
        &client,
        &app.address,
        &format!("note/{}/next", loc1.get_id()),
        "next title",
        "New body text",
        "",
    )
    .await;

    let response = get_note_helper(&client, &app.address, &loc1).await;
    assert_eq!(response["next"].as_str().unwrap(), loc2.get_id().as_ref());

    let response = get_note_helper(&client, &app.address, &loc2).await;
    assert_eq!(response["prev"].as_str().unwrap(), loc1.get_id().as_ref());
}

#[tokio::test]
async fn orphan_reference() {
    let app = spawn_app().await;
    let client = Client::new();

    let loc1 = create_note_helper(&client, &app.address, "foo", "Fizz", "").await;
    let loc2 = create_note_helper(
        &client,
        &app.address,
        "bar",
        &format!("[here is a link to foo](notegraf:/note/{})", loc1.get_id()),
        "",
    )
    .await;

    let response = client
        .get(&format!("{}/api/v1/note", &app.address))
        .query(&[("query", "!orphan")])
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");
    assert_eq!(response.as_array().unwrap().len(), 1);
    assert_eq!(response[0]["id"], loc2.get_id().as_ref());
}

#[tokio::test]
async fn orphan_prev() {
    let app = spawn_app().await;
    let client = Client::new();

    let loc1 = create_note_helper(&client, &app.address, "title", "## body text", "").await;
    post_note_helper(
        &client,
        &app.address,
        &format!("note/{}/next", loc1.get_id()),
        "next title",
        "New body text",
        "",
    )
    .await;

    let response = client
        .get(&format!("{}/api/v1/note", &app.address))
        .query(&[("query", "!orphan")])
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");
    assert_eq!(response.as_array().unwrap().len(), 1);
    assert_eq!(response[0]["id"], loc1.get_id().as_ref());
}

#[tokio::test]
async fn orphan_parent() {
    let app = spawn_app().await;
    let client = Client::new();

    let loc1 = create_note_helper(&client, &app.address, "title", "## body text", "").await;
    post_note_helper(
        &client,
        &app.address,
        &format!("note/{}/branch", loc1.get_id()),
        "child title",
        "New body text",
        "",
    )
    .await;

    let response = client
        .get(&format!("{}/api/v1/note", &app.address))
        .query(&[("query", "!orphan")])
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");
    assert_eq!(response.as_array().unwrap().len(), 1);
    assert_eq!(response[0]["id"], loc1.get_id().as_ref());
}

#[tokio::test]
async fn tags() {
    let app = spawn_app().await;
    let client = Client::new();

    create_note_helper(&client, &app.address, "foo", "", "tag1").await;
    create_note_helper(&client, &app.address, "foo", "", "tag2").await;

    let response = client
        .get(&format!("{}/api/v1/tags", &app.address))
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");
    let tags = response.as_array().unwrap();
    assert_eq!(tags.len(), 2);
    assert!(tags.contains(&json! {"tag1"}));
    assert!(tags.contains(&json! {"tag2"}));
}

#[tokio::test]
async fn limit_override() {
    let app = spawn_app().await;
    let client = Client::new();

    let _loc1 = create_note_helper(&client, &app.address, "title", "## body text", "").await;
    let loc2 = create_note_helper(&client, &app.address, "title2", "## body text", "").await;

    let response = client
        .get(&format!("{}/api/v1/note", &app.address))
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");
    assert_eq!(response.as_array().unwrap().len(), 2);

    // recent note comes first
    let response = client
        .get(&format!("{}/api/v1/note", &app.address))
        .query(&[("query", "!limit=1")])
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");
    assert_eq!(response.as_array().unwrap().len(), 1);
    assert_eq!(response[0]["id"], loc2.get_id().as_ref());
}

#[tokio::test]
async fn tag_exclude() {
    let app = spawn_app().await;
    let client = Client::new();

    let loc1 = create_note_helper(&client, &app.address, "foo", "", "tag1").await;
    let loc2 = create_note_helper(&client, &app.address, "foo", "", "tag2").await;

    let response = client
        .get(&format!("{}/api/v1/note", &app.address))
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");
    assert_eq!(response.as_array().unwrap().len(), 2);

    let response = client
        .get(&format!("{}/api/v1/note", &app.address))
        .query(&[("query", "foo")])
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");
    assert_eq!(response.as_array().unwrap().len(), 2);

    let response = client
        .get(&format!("{}/api/v1/note", &app.address))
        .query(&[("query", "foo -#tag1")])
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");
    assert_eq!(response.as_array().unwrap().len(), 1);
    assert_eq!(response[0]["id"], loc2.get_id().as_ref());

    let response = client
        .get(&format!("{}/api/v1/note", &app.address))
        .query(&[("query", "foo -#tag2")])
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");
    assert_eq!(response.as_array().unwrap().len(), 1);
    assert_eq!(response[0]["id"], loc1.get_id().as_ref());
}

#[tokio::test]
async fn lexeme_exclude() {
    let app = spawn_app().await;
    let client = Client::new();

    let loc1 = create_note_helper(&client, &app.address, "hello world", "", "tag1").await;
    let loc2 = create_note_helper(&client, &app.address, "goodbye world", "", "tag2").await;

    let response = client
        .get(&format!("{}/api/v1/note", &app.address))
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");
    assert_eq!(response.as_array().unwrap().len(), 2);

    let response = client
        .get(&format!("{}/api/v1/note", &app.address))
        .query(&[("query", "world")])
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");
    assert_eq!(response.as_array().unwrap().len(), 2);

    let response = client
        .get(&format!("{}/api/v1/note", &app.address))
        .query(&[("query", "world -hello")])
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");
    assert_eq!(response.as_array().unwrap().len(), 1);
    assert_eq!(response[0]["id"], loc2.get_id().as_ref());

    let response = client
        .get(&format!("{}/api/v1/note", &app.address))
        .query(&[("query", "world -goodbye")])
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");
    assert_eq!(response.as_array().unwrap().len(), 1);
    assert_eq!(response[0]["id"], loc1.get_id().as_ref());
}

#[tokio::test]
async fn issue_158() {
    let app = spawn_app().await;
    let client = Client::new();

    let loc1 = create_note_helper(&client, &app.address, "hello world", "", "tag1").await;
    let loc2 = create_note_helper(&client, &app.address, "goodbye world", "", "tag2").await;

    let response = client
        .get(&format!("{}/api/v1/note", &app.address))
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");
    assert_eq!(response.as_array().unwrap().len(), 2);

    let response = client
        .get(&format!("{}/api/v1/note", &app.address))
        .query(&[("query", "world")])
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");
    assert_eq!(response.as_array().unwrap().len(), 2);

    let response = client
        .get(&format!("{}/api/v1/note", &app.address))
        .query(&[("query", "-hello")])
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");
    assert_eq!(response.as_array().unwrap().len(), 1);
    assert_eq!(response[0]["id"], loc2.get_id().as_ref());

    let response = client
        .get(&format!("{}/api/v1/note", &app.address))
        .query(&[("query", "-goodbye")])
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<Value>()
        .await
        .expect("Failed to parse response");
    assert_eq!(response.as_array().unwrap().len(), 1);
    assert_eq!(response[0]["id"], loc1.get_id().as_ref());
}
