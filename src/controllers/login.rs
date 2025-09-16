use actix_web::{web, HttpResponse, Error};
use actix_session::Session;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, DatabaseConnection};
use crate::models::user;
use crate::LoginForm;
// use bcrypt::{hash, verify, DEFAULT_COST};
use bcrypt::verify;

// Serve login form (GET)
pub async fn login_form() -> HttpResponse {
    let template = include_str!("../views/login/login_form.html");
    let html = template.replace("{{error}}", "");
    HttpResponse::Ok().content_type("text/html").body(html)
}


// Helper: verify password
pub fn verify_password(hash: &str, password: &str) -> bool {
    verify(password, hash).unwrap_or(false)
}

// Login handler
pub async fn login(
    db: web::Data<DatabaseConnection>, // Use DatabaseConnection directly
    session: Session,
    form: web::Form<LoginForm>,
) -> Result<HttpResponse, Error> {
    let username = form.username.clone();
    let password = form.password.clone();

    let user = user::Entity::find()
        .filter(user::Column::Username.eq(username.clone()))
        .one(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    if let Some(u) = user {
        if verify_password(&u.password_hash, &password) {
            session.insert("user_id", u.id)?;
            return Ok(HttpResponse::Found().append_header(("Location", "/")).finish());
        }
    }

    Ok(HttpResponse::Unauthorized().body("Invalid credentials"))
}

// Logout handler
pub async fn logout(session: Session) -> Result<HttpResponse, Error> {
    session.purge();
    Ok(HttpResponse::Found().append_header(("Location", "/")).finish())
}

// Check if logged in
pub fn is_logged_in(session: &Session) -> bool {
    session.get::<i32>("user_id").unwrap_or(None).is_some()
}

pub fn login_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/login", web::get().to(login_form))
        .route("/login", web::post().to(login))
        .route("/logout", web::post().to(|session: Session| async move {
            logout(session).await
        }));
}