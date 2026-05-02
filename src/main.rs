use std::env;

use axum::{Json, Router, extract::{Path, State}, http::StatusCode, response::IntoResponse, routing::{delete, get}};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Result, query_as};
use tokio::net::TcpListener;

#[derive(Clone)]
struct AppState {
    pool: PgPool
}

#[derive(Serialize, Deserialize, FromRow)]
struct User {
    id: i32,
    name: String,
}

#[derive(Deserialize)]
struct RequestUser {
    name: String
}

async fn list_users(
    State(state): State<AppState>
) -> impl IntoResponse {
    let query = "SELECT * FROM items";

    match query_as::<_, User>(query)
        .fetch_all(&state.pool)
        .await
    {
        Ok(users) => (StatusCode::OK, Json(users)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i32>
) -> impl IntoResponse {
    let query = r#"
        SELECT * FROM items
        WHERE id = $1
    "#;

    match query_as::<_, User>(query)
        .bind(id)
        .fetch_optional(&state.pool)
        .await
    {
        Ok(Some(user)) => (StatusCode::OK, Json(user)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

async fn create_user(
    State(state): State<AppState>,
    Json(body): Json<RequestUser>
) -> impl IntoResponse {
    let query = r#"
        INSERT INTO items (name)
        VALUES ($1)
        RETURNING id, name
    "#;

    match query_as::<_, User>(query)
        .bind(body.name)
        .fetch_one(&state.pool)
        .await
    {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let query = r#"
        DELETE FROM items
        WHERE id = $1
    "#;

    match sqlx::query(query)
        .bind(id)
        .execute(&state.pool)
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                StatusCode::NO_CONTENT.into_response()
            } else {
                StatusCode::NOT_FOUND.into_response()
            }
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(body): Json<RequestUser>
) -> impl IntoResponse {
    let query = r#"
        UPDATE items
        SET name = $1
        WHERE id = $2
        RETURNING id, name
    "#;

    match query_as::<_, User>(query)
        .bind(&body.name)
        .bind(id)
        .fetch_optional(&state.pool)
        .await
    {
        Ok(Some(user)) => (StatusCode::OK, Json(user)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv::dotenv().ok();

    let db_url = env::var("DATABASE_URL").unwrap();
    let pool = PgPool::connect(&db_url).await?;

    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .route("/users",
            get(list_users)
            .post(create_user)
        )
        .route("/users/{id}",
            delete(delete_user)
            .get(get_user)
            .put(update_user)
        )
        .with_state(AppState {pool});

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;

    println!("Listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
