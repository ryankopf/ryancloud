use actix_session::Session;
use actix_web::{web, HttpResponse, Error, HttpRequest};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use crate::models::user;
use argon2::{self, Config};

// Helper: hash password
pub fn hash_password(password: &str) -> String {
    let salt = b"randomsalt";
    argon2::hash_encoded(password.as_bytes(), salt, &Config::default()).unwrap()
}

// Helper: verify password
pub fn verify_password(hash: &str, password: &str) -> bool {
    argon2::verify_encoded(hash, password.as_bytes()).unwrap_or(false)
}

// Login handler
pub async fn login(
    db: web::Data<DatabaseConnection>,
    session: Session,
    form: web::Form<(String, String)>, // (username, password)
) -> Result<HttpResponse, Error> {
    let (username, password) = form.into_inner();
    let user = user::Entity::find()
        .filter(user::Column::Username.eq(username.clone()))
        .one(db.get_ref())
        .await
        .map_err(|_| HttpResponse::InternalServerError())?;
    if let Some(u) = user {
        if verify_password(&u.password_hash, &password) {
            session.insert("user_id", u.id)?;
            return Ok(HttpResponse::Found().header("Location", "/").finish());
        }
    }
    Ok(HttpResponse::Unauthorized().body("Invalid credentials"))
}

// Logout handler
pub async fn logout(session: Session) -> Result<HttpResponse, Error> {
    session.purge();
    Ok(HttpResponse::Found().header("Location", "/").finish())
}

// Check if logged in
pub fn is_logged_in(session: &Session) -> bool {
    session.get::<i32>("user_id").unwrap_or(None).is_some()
}
