use std::env;

use axum::{Json, Router, extract::{Path, State}, http::StatusCode, response::IntoResponse, routing::{get}};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Result, query_as};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    pool: PgPool
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
struct Task {
    id: i32,
    description: String,
    status: String,
    created_at: NaiveDateTime
}

#[derive(Deserialize)]
struct CreateTask {
    description: String,
}

#[derive(Deserialize)]
struct UpdateTask {
    description: String,
    status: String,
}

async fn list_tasks(
    State(state): State<AppState>
) -> impl IntoResponse {
    let query = "SELECT * FROM items";

    match query_as::<_, Task>(query)
        .fetch_all(&state.pool)
        .await
    {
        Ok(tasks) => (StatusCode::OK, Json(tasks)).into_response(),
        Err(e) => {
            eprintln!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response() 
        }
    }
}

async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<i32>
) -> impl IntoResponse {
    let query = r#"
        SELECT * FROM items
        WHERE id = $1
    "#;

    match query_as::<_, Task>(query)
        .bind(id)
        .fetch_optional(&state.pool)
        .await
    {
        Ok(Some(task)) => (StatusCode::OK, Json(task)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response() 
        }
    }
}

async fn create_task(
    State(state): State<AppState>,
    Json(body): Json<CreateTask>
) -> impl IntoResponse {
    let query = r#"
        INSERT INTO items (description)
        VALUES ($1)
        RETURNING id, description, status, created_at
    "#;

    match query_as::<_, Task>(query)
        .bind(body.description)
        .fetch_one(&state.pool)
        .await
    {
        Ok(task) => (StatusCode::CREATED, Json(task)).into_response(),
        Err(e) => {
            eprintln!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn delete_task(
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
        Err(e) => {
            eprintln!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn update_task(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(body): Json<UpdateTask>
) -> impl IntoResponse {
    let query = r#"
        UPDATE items
        SET description = $1, status = $2
        WHERE id = $3
        RETURNING id, description, status, created_at
    "#;

    match query_as::<_, Task>(query)
        .bind(&body.description)
        .bind(&body.status)
        .bind(id)
        .fetch_optional(&state.pool)
        .await
    {
        Ok(Some(task)) => (StatusCode::OK, Json(task)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv::dotenv().ok();

    let db_url = env::var("DATABASE_URL").unwrap();
    let pool = PgPool::connect(&db_url).await?;

    let start_query = r#"
        CREATE TABLE IF NOT EXISTS items (
            id SERIAL PRIMARY KEY,
            description TEXT,
            status VARCHAR(50) NOT NULL DEFAULT 'in_progress' CHECK (status IN ('in_progress', 'done')),
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
    "#;

    sqlx::query(start_query)
        .execute(&pool)
        .await?;

    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .route("/tasks",
            get(list_tasks)
            .post(create_task)
        )
        .route("/tasks/{id}",
            get(get_task)
            .put(update_task)
            .delete(delete_task)
        )
        .with_state(AppState {pool})
        .layer(CorsLayer::permissive());

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;

    println!("Listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
