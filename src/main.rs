mod db;
mod session;
mod sla;

use axum::{
    extract::{Form, Path, State},
    response::{Html, Redirect, Response, IntoResponse},
    routing:: get,
    Router,
};
use axum_extra::extract::cookie::CookieJar;
use bcrypt::{hash, verify};
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

#[derive(Deserialize)]
struct TicketUpdateForm {
    status: String,
}

#[derive(Deserialize)]
struct GuestTicketForm {
    name: String,
    subject: String,
    description: String,
    priority: String,
    category: String,
}

#[derive(Deserialize)]
struct AccountTicketForm {
    subject: String,
    description: String,
    priority: String,
    category: String,
}

#[derive(Deserialize)]
struct CustomerSignupForm {
    name: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct CustomerLoginForm {
    email: String,
    password: String,
}

fn render_template(
    state: &AppState,
    template_name: &str,
    context: &Context,
) -> Result<Html<String>, (axum::http::StatusCode, String)> {
    state
        .templates
        .render(template_name, context)
        .map(Html)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

// ===== Staff auth handlers =====

async fn login_page(State(state): State<AppState>) -> Result<Html<String>, (axum::http::StatusCode, String)> {
    let mut context = Context::new();
    context.insert("error", &None::<String>);
    render_template(&state, "login.html", &context)
}

async fn login_submit(
    State(state): State<AppState>,
    _jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> Result<Response, Response> {
    let pool = &state.pool;

    let user = sqlx::query_as::<_, (i64, String, String)>(
        "SELECT id, username, password FROM users WHERE username = ?"
    )
    .bind(&form.username)
    .fetch_optional(pool)
    .await
    .map_err(|_| {
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

// ===== Customer auth handlers =====

async fn customer_signup_page(State(state): State<AppState>) -> Result<Html<String>, (axum::http::StatusCode, String)> {
    let mut context = Context::new();
    context.insert("error", &None::<String>);
    render_template(&state, "customer_signup.html", &context)
}

async fn customer_signup_submit(
    State(state): State<AppState>,
    _jar: CookieJar,
    Form(form): Form<CustomerSignupForm>,
) -> Result<Response, Response> {
    let pool = &state.pool;

    let existing = sqlx::query_as::<_, (i64,)>("SELECT id FROM customers WHERE email = ?")
        .bind(&form.email)
        .fetch_optional(pool)
        .await
        .map_err(|_| {
            let mut context = Context::new();
            context.insert("error", &"Database error");
            let body = state.templates.render("customer_signup.html", &context).unwrap();
            Response::builder()
                .status(500)
                .header("content-type", "text/html")
                .body(axum::body::Body::from(body))
                .unwrap()
        })?;

    if existing.is_some() {
        let mut context = Context::new();
        context.insert("error", &"An account with that email already exists.");
        let body = state.templates.render("customer_signup.html", &context).unwrap();
        return Err(Response::builder()
            .status(200)
            .header("content-type", "text/html")
            .body(axum::body::Body::from(body))
            .unwrap());
    }

    let hashed_password = hash(&form.password, 10).unwrap();
    let result = sqlx::query("INSERT INTO customers (name, email, password) VALUES (?, ?, ?)")
        .bind(&form.name)
        .bind(&form.email)
        .bind(&hashed_password)
        .execute(pool)
        .await
        .map_err(|_| {
            let mut context = Context::new();
            context.insert("error", &"Signup failed");
            let body = state.templates.render("customer_signup.html", &context).unwrap();
            Response::builder()
                .status(500)
                .header("content-type", "text/html")
                .body(axum::body::Body::from(body))
                .unwrap()
        })?;

    let customer_id = result.last_insert_rowid();
    let token = session::create_customer_session(pool, customer_id)
        .await
        .map_err(|_| {
            let mut context = Context::new();
            context.insert("error", &"Session creation failed");
            let body = state.templates.render("customer_signup.html", &context).unwrap();
            Response::builder()
                .status(500)
                .header("content-type", "text/html")
                .body(axum::body::Body::from(body))
                .unwrap()
        })?;

    let mut response = Redirect::to("/new/account").into_response();
    session::set_session_cookie(&mut response, &token);
    Ok(response)
}

async fn customer_login_page(State(state): State<AppState>) -> Result<Html<String>, (axum::http::StatusCode, String)> {
    let mut context = Context::new();
    context.insert("error", &None::<String>);
    render_template(&state, "customer_login.html", &context)
}

async fn customer_login_submit(
    State(state): State<AppState>,
    _jar: CookieJar,
    Form(form): Form<CustomerLoginForm>,
) -> Result<Response, Response> {
    let pool = &state.pool;

    let customer = sqlx::query_as::<_, (i64, String, String, String)>(
        "SELECT id, name, email, password FROM customers WHERE email = ?"
    )
    .bind(&form.email)
    .fetch_optional(pool)
    .await
    .map_err(|_| {
        let mut context = Context::new();
        context.insert("error", &"Database error");
        let body = state.templates.render("customer_login.html", &context).unwrap();
        Response::builder()
            .status(500)
            .header("content-type", "text/html")
            .body(axum::body::Body::from(body))
            .unwrap()
    })?;

    let customer = match customer {
        Some(c) => c,
        None => {
            let mut context = Context::new();
            context.insert("error", &"Invalid email or password");
            let body = state.templates.render("customer_login.html", &context).unwrap();
            return Err(Response::builder()
                .status(200)
                .header("content-type", "text/html")
                .body(axum::body::Body::from(body))
                .unwrap());
        }
    };

    if !verify(&form.password, &customer.3).unwrap_or(false) {
        let mut context = Context::new();
        context.insert("error", &"Invalid email or password");
        let body = state.templates.render("customer_login.html", &context).unwrap();
        return Err(Response::builder()
            .status(200)
            .header("content-type", "text/html")
            .body(axum::body::Body::from(body))
            .unwrap());
    }

    let token = session::create_customer_session(pool, customer.0)
        .await
        .map_err(|_| {
            let mut context = Context::new();
            context.insert("error", &"Session creation failed");
            let body = state.templates.render("customer_login.html", &context).unwrap();
            Response::builder()
                .status(500)
                .header("content-type", "text/html")
                .body(axum::body::Body::from(body))
                .unwrap()
        })?;

    let mut response = Redirect::to("/new/account").into_response();
    session::set_session_cookie(&mut response, &token);
    Ok(response)
}

async fn customer_logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Response {
    if let Some(cookie) = jar.get("session") {
        let token = cookie.value();
        let _ = session::destroy_session(&state.pool, token).await;
    }
    let mut response = Redirect::to("/").into_response();
    session::remove_session_cookie(&mut response);
    response
}

// ===== Ticket submission =====

async fn new_choice_page(State(state): State<AppState>) -> Result<Html<String>, (axum::http::StatusCode, String)> {
    let context = Context::new();
    render_template(&state, "new_choice.html", &context)
}

async fn new_guest_page(State(state): State<AppState>) -> Result<Html<String>, (axum::http::StatusCode, String)> {
    let context = Context::new();
    render_template(&state, "new_guest.html", &context)
}

async fn new_guest_submit(
    State(state): State<AppState>,
    Form(form): Form<GuestTicketForm>,
) -> Result<Html<String>, (axum::http::StatusCode, String)> {
    let pool = &state.pool;
    let result = sqlx::query(
        "INSERT INTO tickets (subject, description, priority, category, submitted_by, customer_id) VALUES (?, ?, ?, ?, ?, NULL)"
    )
    .bind(&form.subject)
    .bind(&form.description)
    .bind(&form.priority)
    .bind(&form.category)
    .bind(&form.name)
    .execute(pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let ticket_id = result.last_insert_rowid();
    let mut context = Context::new();
    context.insert("ticket_id", &ticket_id);
    context.insert("subject", &form.subject);
    render_template(&state, "ticket_confirmation.html", &context)
}

async fn new_account_page(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Html<String>, (axum::http::StatusCode, String)> {
    if let Some(cookie) = jar.get("session") {
        let token = cookie.value();
        if let Ok(Some(customer_id)) = session::get_customer_id(&state.pool, token).await {
            let customer = sqlx::query_as::<_, (i64, String, String, String)>(
                "SELECT id, name, email, password FROM customers WHERE id = ?"
            )
            .bind(customer_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            if let Some(customer) = customer {
                let mut context = Context::new();
                context.insert("customer", &serde_json::json!({
                    "name": customer.1,
                }));
                return render_template(&state, "new_account.html", &context);
            }
        }
    }
    Ok(Html("<script>window.location='/account/login'</script>".to_string()))
}

async fn new_account_submit(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<AccountTicketForm>,
) -> Result<Html<String>, (axum::http::StatusCode, String)> {
    if let Some(cookie) = jar.get("session") {
        let token = cookie.value();
        if let Ok(Some(customer_id)) = session::get_customer_id(&state.pool, token).await {
            let customer = sqlx::query_as::<_, (String,)>("SELECT name FROM customers WHERE id = ?")
                .bind(customer_id)
                .fetch_optional(&state.pool)
                .await
                .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            if let Some((name,)) = customer {
                let result = sqlx::query(
                    "INSERT INTO tickets (subject, description, priority, category, submitted_by, customer_id) VALUES (?, ?, ?, ?, ?, ?)"
                )
                .bind(&form.subject)
                .bind(&form.description)
                .bind(&form.priority)
                .bind(&form.category)
                .bind(&name)
                .bind(customer_id)
                .execute(&state.pool)
                .await
                .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

                let ticket_id = result.last_insert_rowid();
                let mut context = Context::new();
                context.insert("ticket_id", &ticket_id);
                context.insert("subject", &form.subject);
                return render_template(&state, "ticket_confirmation.html", &context);
            }
        }
    }
    Ok(Html("<script>window.location='/account/login'</script>".to_string()))
}

// ===== Home and staff area =====

async fn home_page(State(state): State<AppState>) -> Result<Html<String>, (axum::http::StatusCode, String)> {
    let pool = &state.pool;

    let tickets: Vec<(i64, String, String, String, String, String, Option<i64>, String, Option<String>)> =
        sqlx::query_as(
            "SELECT id, subject, description, priority, category, status, customer_id, opened_at, resolved_at FROM tickets"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let total_count = tickets.len();
    let resolved_count = tickets.iter().filter(|t| t.5 == "Resolved").count();
    let open_count = total_count - resolved_count;

    let circumference = 2.0 * std::f64::consts::PI * 45.0;
    let resolved_percent = if total_count > 0 { (resolved_count as f64 / total_count as f64) * 100.0 } else { 0.0 };
    let open_percent = if total_count > 0 { (open_count as f64 / total_count as f64) * 100.0 } else { 0.0 };
    let resolved_arc = (resolved_percent / 100.0) * circumference;
    let open_arc = (open_percent / 100.0) * circumference;
    let resolved_arc_neg = -resolved_arc;

    let mut context = Context::new();
    context.insert("total_count", &total_count);
    context.insert("resolved_count", &resolved_count);
    context.insert("open_count", &open_count);
    context.insert("circumference", &circumference);
    context.insert("resolved_arc", &resolved_arc);
    context.insert("open_arc", &open_arc);
    context.insert("resolved_arc_neg", &resolved_arc_neg);

    render_template(&state, "home.html", &context)
}

async fn staff_dashboard(State(state): State<AppState>, jar: CookieJar) -> Response {
    if let Some(cookie) = jar.get("session") {
        let token = cookie.value();
        if let Ok(Some(_)) = session::get_staff_user_id(&state.pool, token).await {
            let tickets: Vec<(i64, String, String, String, String, String, Option<i64>, String, Option<String>)> =
                match sqlx::query_as(
                    "SELECT id, subject, description, priority, category, status, customer_id, opened_at, resolved_at FROM tickets ORDER BY opened_at DESC"
                )
                .fetch_all(&state.pool)
                .await
                {
                    Ok(rows) => rows,
                    Err(_) => return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
                };

            let mut context = Context::new();
            let mut ticket_views = Vec::new();
            for t in &tickets {
                let sla = sla::get_sla_status(&t.3, &t.5, &t.7);
                ticket_views.push(serde_json::json!({
                    "id": t.0,
                    "subject": t.1,
                    "description": t.2,
                    "priority": t.3,
                    "category": t.4,
                    "status": t.5,
                    "customer_id": t.6,
                    "opened_at": t.7,
                    "resolved_at": t.8,
                    "sla": {
                        "label": sla.label,
                        "color": sla.color,
                        "bg": sla.bg,
                    }
                }));
            }
            context.insert("tickets", &ticket_views);
            return match render_template(&state, "tickets.html", &context) {
                Ok(html) => html.into_response(),
                Err((status, msg)) => (status, msg).into_response(),
            };
        }
    }
    Redirect::to("/login").into_response()
}

async fn ticket_detail(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<i64>,
) -> Response {
    if let Some(cookie) = jar.get("session") {
        let token = cookie.value();
        if let Ok(Some(_)) = session::get_staff_user_id(&state.pool, token).await {
            let ticket: Option<(i64, String, String, String, String, String, Option<i64>, String, Option<String>)> =
                sqlx::query_as(
                    "SELECT id, subject, description, priority, category, status, customer_id, opened_at, resolved_at FROM tickets WHERE id = ?"
                )
                .bind(id)
                .fetch_optional(&state.pool)
                .await
                .unwrap_or(None);

            if ticket.is_none() {
                return Redirect::to("/staff").into_response();
            }
            let t = ticket.unwrap();

            let mut customer_email = None;
            if let Some(customer_id) = t.6 {
                if let Ok(Some((email,))) = sqlx::query_as::<_, (String,)>("SELECT email FROM customers WHERE id = ?")
                    .bind(customer_id)
                    .fetch_optional(&state.pool)
                    .await
                {
                    customer_email = Some(email);
                }
            }

            let sla_status = sla::get_sla_status(&t.3, &t.5, &t.7);
            let mut context = Context::new();
            context.insert("ticket", &serde_json::json!({
                "id": t.0,
                "subject": t.1,
                "description": t.2,
                "priority": t.3,
                "category": t.4,
                "status": t.5,
                "customer_id": t.6,
                "opened_at": t.7,
                "resolved_at": t.8,
            }));
            context.insert("customer_email", &customer_email);
            context.insert("sla", &serde_json::json!({
                "label": sla_status.label,
                "color": sla_status.color,
                "bg": sla_status.bg,
            }));

            return match render_template(&state, "ticket_detail.html", &context) {
                Ok(html) => html.into_response(),
                Err((status, msg)) => (status, msg).into_response(),
            };
        }
    }
    Redirect::to("/login").into_response()
}

async fn ticket_update(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<i64>,
    Form(form): Form<TicketUpdateForm>,
) -> Response {
    if let Some(cookie) = jar.get("session") {
        let token = cookie.value();
        if let Ok(Some(_)) = session::get_staff_user_id(&state.pool, token).await {
            let resolved_at = if form.status == "Resolved" {
                Some(chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string())
            } else {
                None
            };

            let result = sqlx::query("UPDATE tickets SET status = ?, resolved_at = ? WHERE id = ?")
                .bind(&form.status)
                .bind(&resolved_at)
                .bind(id)
                .execute(&state.pool)
                .await;

            if result.is_ok() {
                return Redirect::to(&format!("/ticket/{}", id)).into_response();
            }
        }
    }
    Redirect::to("/login").into_response()
}

#[tokio::main]
async fn main() {
    let pool = db::init_pool().await.expect("Failed to initialize database");
    let templates = Tera::new("templates/**/*.html").expect("Failed to load templates");
    let state = AppState { pool, templates };

    let app = Router::new()
        .route("/", get(home_page))
        .route("/login", get(login_page).post(login_submit))
        .route("/logout", get(logout))
        .route("/staff", get(staff_dashboard))
        .route("/ticket/:id", get(ticket_detail).post(ticket_update))
        .route("/new", get(new_choice_page))
        .route("/new/guest", get(new_guest_page).post(new_guest_submit))
        .route("/new/account", get(new_account_page).post(new_account_submit))
        .route("/account/signup", get(customer_signup_page).post(customer_signup_submit))
        .route("/account/login", get(customer_login_page).post(customer_login_submit))
        .route("/account/logout", get(customer_logout))
        .fallback_service(ServeDir::new("public"))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server running at http://localhost:3000");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
