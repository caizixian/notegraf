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
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, db_name))
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
    let store: PostgreSQLStore<PlainNote> = get_store().await;
    let loc1 = store
        .new_note("".to_owned(), PlainNote::new("Note".into()), None)
        .await
        .unwrap();
    assert!(!store
        .is_deleted(loc1.get_id().try_to_uuid().unwrap())
        .await
        .unwrap());
    store.delete_note(&loc1).await.unwrap();
    assert!(store
        .is_deleted(loc1.get_id().try_to_uuid().unwrap())
        .await
        .unwrap());
}

#[tokio::test]
async fn delete_note_current() {
    let store: PostgreSQLStore<PlainNote> = get_store().await;
    let loc1 = store
        .new_note("".to_owned(), PlainNote::new("Note".into()), None)
        .await
        .unwrap();
    store.delete_note(&loc1.current()).await.unwrap();
    assert!(store
        .is_deleted(loc1.get_id().try_to_uuid().unwrap())
        .await
        .unwrap());
}

#[tokio::test]
async fn delete_note_with_branches() {
    common_tests::delete_note_with_branches(get_store().await).await;
}

#[tokio::test]
async fn delete_middle_note_sequence() {
    common_tests::delete_middle_note_sequence(get_store().await).await;
}