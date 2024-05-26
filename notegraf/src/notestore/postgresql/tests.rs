use super::*;
use crate::notestore::tests as common_tests;
use crate::notetype::PlainNote;
use sqlx::{Connection, Executor, PgConnection};
use std::env;

/// Configure the connect options with the following environment variables
///
/// NOTEGRAF_DATABASE_HOST: default "localhost"
/// NOTEGRAF_DATABASE_PORT: default "5432"
/// NOTEGRAF_DATABASE_USERNAME: default not set
/// NOTEGRAF_DATABASE_PASSWORD: default not set
fn get_connect_options() -> PgConnectOptions {
    let host = env::var("NOTEGRAF_DATABASE_HOST").unwrap_or("localhost".to_owned());
    let port = env::var("NOTEGRAF_DATABASE_PORT").unwrap_or("5432".to_owned());
    let username = env::var("NOTEGRAF_DATABASE_USERNAME");
    let password = env::var("NOTEGRAF_DATABASE_PASSWORD");
    let options = PgConnectOptions::new()
        .host(&host)
        .port(port.parse().expect("Failed to parse port number"));
    if let Ok(ref u) = username {
        let p = password
            .as_ref()
            .expect("Password expected when a username is set");
        options.username(u).password(p)
    } else {
        options
    }
}

async fn get_store() -> PostgreSQLStore<PlainNote> {
    let options = get_connect_options();
    let mut connection = PgConnection::connect_with(&options)
        .await
        .expect("Failed to connect to Postgres");
    let db_name = Uuid::new_v4().to_string();
    connection
        .execute(&*format!(r#"CREATE DATABASE "{db_name}";"#))
        .await
        .expect("Failed to create database.");
    PostgreSQLStoreBuilder::new(options.database(&db_name))
        .build()
        .await
}

#[tokio::test]
async fn unique_id() {
    common_tests::unique_id(get_store().await).await;
}

#[tokio::test]
async fn new_note_revision() {
    common_tests::new_note_revision(get_store().await).await;
}

#[tokio::test]
async fn new_note_retrieve() {
    common_tests::new_note_retrieve(get_store().await).await;
}

#[tokio::test]
async fn update_note() {
    common_tests::update_note(get_store().await).await;
}

#[tokio::test]
async fn add_branch() {
    common_tests::add_branch(get_store().await).await;
}

#[tokio::test]
async fn delete_note_specific() {
    common_tests::delete_note_specific(get_store().await).await;
}

#[tokio::test]
async fn delete_note_current() {
    common_tests::delete_note_current(get_store().await).await;
}

#[tokio::test]
async fn delete_note_with_branches() {
    common_tests::delete_note_with_branches(get_store().await).await;
}

#[tokio::test]
async fn delete_child() {
    common_tests::delete_child(get_store().await).await;
}

#[tokio::test]
async fn delete_child_sequence_top() {
    common_tests::delete_child_sequence_top(get_store().await).await;
}

#[tokio::test]
async fn resurrect_deleted_note() {
    common_tests::resurrect_deleted_note(get_store().await).await;
}

#[tokio::test]
async fn delete_first_note_sequence() {
    common_tests::delete_first_note_sequence(get_store().await).await;
}

#[tokio::test]
async fn delete_last_note_sequence() {
    common_tests::delete_last_note_sequence(get_store().await).await;
}

#[tokio::test]
async fn delete_middle_note_sequence() {
    common_tests::delete_middle_note_sequence(get_store().await).await;
}

#[tokio::test]
async fn resurrect_note_in_sequence() {
    common_tests::resurrect_note_in_sequence(get_store().await).await;
}

#[tokio::test]
async fn search_recent() {
    common_tests::search_recent(get_store().await).await;
}

#[tokio::test]
async fn search_fulltext() {
    common_tests::search_fulltext(get_store().await).await;
}

#[tokio::test]
async fn search_nonexist() {
    common_tests::search_nonexist(get_store().await).await;
}

#[tokio::test]
async fn backlink() {
    common_tests::backlink(get_store().await).await;
}

#[tokio::test]
async fn search_tags() {
    common_tests::search_tags(get_store().await).await;
}

#[tokio::test]
async fn search_orphan() {
    common_tests::search_orphan(get_store().await).await;
}

#[tokio::test]
async fn search_notag() {
    common_tests::search_notag(get_store().await).await;
}

#[tokio::test]
async fn issue_151() {
    common_tests::issue_151(get_store().await).await;
}

#[tokio::test]
async fn tags() {
    common_tests::tags(get_store().await).await;
}

#[tokio::test]
async fn search_limit_override() {
    common_tests::search_limit_override(get_store().await).await;
}

#[tokio::test]
async fn search_tag_exclude() {
    common_tests::search_tag_exclude(get_store().await).await;
}

#[tokio::test]
async fn search_lexeme_exclude() {
    common_tests::search_lexeme_exclude(get_store().await).await;
}

#[tokio::test]
async fn issue_158() {
    common_tests::issue_158(get_store().await).await;
}
