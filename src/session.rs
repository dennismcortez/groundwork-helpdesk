use axum::{
    http::header::SET_COOKIE,
    response::Response,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use sqlx::SqlitePool;
use rand::{distributions::Alphanumeric, Rng};
use cookie::time::Duration;

pub fn generate_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

pub async fn create_staff_session(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<String, sqlx::Error> {
    let token = generate_token();
    sqlx::query(
        "INSERT INTO sessions (token, user_id, customer_id, created_at) VALUES (?, ?, NULL, CURRENT_TIMESTAMP)"
    )
    .bind(&token)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(token)
}

pub async fn destroy_session(pool: &SqlitePool, token: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE token = ?")
        .bind(token)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_staff_user_id(pool: &SqlitePool, token: &str) -> Result<Option<i64>, sqlx::Error> {
    let row: Option<(i64,)> = sqlx::query_as("SELECT user_id FROM sessions WHERE token = ?")
        .bind(token)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.0))
}

pub fn set_session_cookie(response: &mut Response, token: &str) {
    let cookie = Cookie::build(("session", token.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build();
    response.headers_mut().insert(
        SET_COOKIE,
        cookie.to_string().parse().unwrap(),
    );
}

pub fn remove_session_cookie(response: &mut Response) {
    let cookie = Cookie::build(("session", ""))
        .path("/")
        .http_only(true)
        .max_age(Duration::seconds(0))
        .build();
    response.headers_mut().insert(
        SET_COOKIE,
        cookie.to_string().parse().unwrap(),
    );
}

pub async fn create_customer_session(
    pool: &SqlitePool,
    customer_id: i64,
) -> Result<String, sqlx::Error> {
    let token = generate_token();
    sqlx::query(
        "INSERT INTO sessions (token, user_id, customer_id, created_at) VALUES (?, NULL, ?, CURRENT_TIMESTAMP)"
    )
    .bind(&token)
    .bind(customer_id)
    .execute(pool)
    .await?;
    Ok(token)
}

pub async fn get_customer_id(pool: &SqlitePool, token: &str) -> Result<Option<i64>, sqlx::Error> {
    let row: Option<(i64,)> = sqlx::query_as("SELECT customer_id FROM sessions WHERE token = ?")
        .bind(token)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.0))
}
