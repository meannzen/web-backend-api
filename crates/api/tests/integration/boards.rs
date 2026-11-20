use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::common::{TestApp, body_json};

async fn create_board(app: &TestApp, name: &str, code: &str) -> serde_json::Value {
    let response = app
        .post_as_admin("/api/v1/boards", &json!({ "name": name, "code": code }))
        .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    body_json(response).await
}

// --- create board ---

#[sqlx::test(migrations = "../../migrations")]
async fn create_board_returns_201_with_uppercased_code(pool: PgPool) {
    let app = TestApp::new(pool);

    let body = create_board(&app, "  Toronto DSB ", " tdsb ").await;

    assert_eq!(body["name"], "Toronto DSB");
    assert_eq!(body["code"], "TDSB");
    assert!(body["id"].is_string());
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_board_returns_409_on_duplicate_code(pool: PgPool) {
    let app = TestApp::new(pool);

    create_board(&app, "Toronto DSB", "TDSB").await;
    let response = app
        .post_as_admin(
            "/api/v1/boards",
            &json!({ "name": "Another board", "code": "tdsb" }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_board_returns_400_on_missing_fields(pool: PgPool) {
    let app = TestApp::new(pool);

    let response = app
        .post_as_admin("/api/v1/boards", &json!({ "name": "", "code": "" }))
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_json(response).await;
    assert!(body["error"]["fields"]["name"].is_array());
    assert!(body["error"]["fields"]["code"].is_array());
}

// --- get board ---

#[sqlx::test(migrations = "../../migrations")]
async fn get_board_returns_200(pool: PgPool) {
    let app = TestApp::new(pool);
    let created = create_board(&app, "Toronto DSB", "TDSB").await;
    let id = created["id"].as_str().unwrap();

    let response = app.get_as_admin(&format!("/api/v1/boards/{id}")).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["id"], id);
    assert_eq!(body["code"], "TDSB");
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_board_returns_404_for_unknown_id(pool: PgPool) {
    let app = TestApp::new(pool);

    let response = app
        .get_as_admin("/api/v1/boards/00000000-0000-0000-0000-000000000000")
        .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// --- list boards ---

#[sqlx::test(migrations = "../../migrations")]
async fn list_boards_returns_empty_list(pool: PgPool) {
    let app = TestApp::new(pool);

    let body = body_json(app.get_as_admin("/api/v1/boards").await).await;

    assert_eq!(body["data"].as_array().unwrap().len(), 0);
    assert_eq!(body["meta"]["has_next_page"], false);
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_boards_default_sort_traverses_all_pages_newest_first(pool: PgPool) {
    let app = TestApp::new(pool);
    for (name, code) in [
        ("Peel DSB", "PDSB"),
        ("Toronto DSB", "TDSB"),
        ("Halton DSB", "HDSB"),
    ] {
        create_board(&app, name, code).await;
    }

    let mut codes = Vec::new();
    let mut uri = "/api/v1/boards?limit=2".to_string();
    loop {
        let body = body_json(app.get_as_admin(&uri).await).await;
        for b in body["data"].as_array().unwrap() {
            codes.push(b["code"].as_str().unwrap().to_string());
        }
        match body["meta"]["next_cursor"].as_str() {
            Some(cursor) => uri = format!("/api/v1/boards?limit=2&after={cursor}"),
            None => break,
        }
    }

    assert_eq!(codes, ["HDSB", "TDSB", "PDSB"]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_boards_returns_400_on_invalid_cursor(pool: PgPool) {
    let app = TestApp::new(pool);

    let response = app.get_as_admin("/api/v1/boards?after=not-a-cursor").await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_boards_sorted_by_name_traverses_all_pages(pool: PgPool) {
    let app = TestApp::new(pool);
    for (name, code) in [
        ("Peel DSB", "PDSB"),
        ("Ottawa-Carleton DSB", "OCDSB"),
        ("Toronto DSB", "TDSB"),
        ("Halton DSB", "HDSB"),
        ("York Region DSB", "YRDSB"),
    ] {
        create_board(&app, name, code).await;
    }

    let mut names = Vec::new();
    let mut uri = "/api/v1/boards?limit=2&sort_by=name&sort_direction=asc".to_string();
    loop {
        let body = body_json(app.get_as_admin(&uri).await).await;
        for b in body["data"].as_array().unwrap() {
            names.push(b["name"].as_str().unwrap().to_string());
        }
        match body["meta"]["next_cursor"].as_str() {
            Some(cursor) => {
                uri =
                    format!("/api/v1/boards?limit=2&sort_by=name&sort_direction=asc&after={cursor}")
            }
            None => break,
        }
    }

    assert_eq!(
        names,
        [
            "Halton DSB",
            "Ottawa-Carleton DSB",
            "Peel DSB",
            "Toronto DSB",
            "York Region DSB"
        ]
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_boards_search_matches_name_or_code(pool: PgPool) {
    let app = TestApp::new(pool);
    create_board(&app, "Toronto DSB", "TDSB").await;
    create_board(&app, "Peel DSB", "PDSB").await;
    create_board(&app, "Halton Catholic", "HCDSB").await;

    let body = body_json(app.get_as_admin("/api/v1/boards?search=toronto").await).await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["code"], "TDSB");

    let body = body_json(app.get_as_admin("/api/v1/boards?search=hcd").await).await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["name"], "Halton Catholic");
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_boards_search_treats_wildcards_literally(pool: PgPool) {
    let app = TestApp::new(pool);
    create_board(&app, "Toronto DSB", "TDSB").await;

    let body = body_json(app.get_as_admin("/api/v1/boards?search=%25").await).await;

    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}
