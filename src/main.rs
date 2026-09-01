mod db;
mod session;

use axum::{
    extract::{Form, State},
    response::{Html, Redirect, Response, IntoResponse},
    routing::{get, post},
    Router,
};
use axum_extra::extract::cookie::CookieJar;
use bcrypt::verify;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tera::{Context, Tera};
use tower_http::services::ServeDir;

#[derive(Clone)]
struct AppState {
    pool: SqlitePool,
    templates: Tera,
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

fn render_template(
    state: &AppState,
    template_name: &str,
    context: &Context,
) -> Result<Html<String>, tera::Error> {
    state.templates.render(template_name, context).map(Html)
}

async fn login_page(State(state): State<AppState>) -> Result<Html<String>, (axum::http::StatusCode, String)> {
    let mut context = Context::new();
    context.insert("error", &None::<String>);
    match render_template(&state, "login.html", &context) {
        Ok(html) => Ok(html),
        Err(e) => Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn login_submit(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> Result<Response, Response> {
    let pool = &state.pool;

    let user = sqlx::query_as::<_, (i64, String, String)>(
        "SELECT id, username, password FROM users WHERE username = ?"
    )
    .bind(&form.username)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        let mut context = Context::new();
        context.insert("error", &"Database error");
        let body = state.templates.render("login.html", &context).unwrap();
        Response::builder()
            .status(500)
            .header("content-type", "text/html")
            .body(axum::body::Body::from(body))
            .unwrap()
    })?;

    let user = match user {
        Some(u) => u,
        None => {
            let mut context = Context::new();
            context.insert("error", &"Invalid username or password");
            let body = state.templates.render("login.html", &context).unwrap();
            return Err(Response::builder()
                .status(200)
                .header("content-type", "text/html")
                .body(axum::body::Body::from(body))
                .unwrap());
        }
    };

    if !verify(&form.password, &user.2).unwrap_or(false) {
        let mut context = Context::new();
        context.insert("error", &"Invalid username or password");
        let body = state.templates.render("login.html", &context).unwrap();
        return Err(Response::builder()
            .status(200)
            .header("content-type", "text/html")
            .body(axum::body::Body::from(body))
            .unwrap());
    }

    let token = session::create_staff_session(pool, user.0)
        .await
        .map_err(|_| {
            let mut context = Context::new();
            context.insert("error", &"Session creation failed");
            let body = state.templates.render("login.html", &context).unwrap();
            Response::builder()
                .status(500)
                .header("content-type", "text/html")
                .body(axum::body::Body::from(body))
                .unwrap()
        })?;

    let mut response = Redirect::to("/staff").into_response();
    session::set_session_cookie(&mut response, &token);
    Ok(response)
}

async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Response {
    if let Some(cookie) = jar.get("session") {
        let token = cookie.value();
        let _ = session::destroy_session(&state.pool, token).await;
    }
    let mut response = Redirect::to("/login").into_response();
    session::remove_session_cookie(&mut response);
    response
}

async fn staff_dashboard(State(state): State<AppState>, jar: CookieJar) -> Response {
    if let Some(cookie) = jar.get("session") {
        let token = cookie.value();
        if let Ok(Some(_)) = session::get_staff_user_id(&state.pool, token).await {
            return Html("<h1>Staff dashboard (coming soon)</h1>").into_response();
        }
    }
    Redirect::to("/login").into_response()
}

#[tokio::main]
async fn main() {
    let pool = db::init_pool().await.expect("Failed to initialize database");
    let templates = Tera::new("templates/**/*").expect("Failed to load templates");
    let state = AppState { pool, templates };

    let app = Router::new()
        .route("/", get(|| async { "Hello from Rust!" }))
        .route("/login", get(login_page).post(login_submit))
        .route("/logout", get(logout))
        .route("/staff", get(staff_dashboard))
        .fallback_service(ServeDir::new("public"))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server running at http://localhost:3000");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
