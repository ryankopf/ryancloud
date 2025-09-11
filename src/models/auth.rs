use actix_session::Session;
use actix_web::{web, HttpResponse, Error};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use crate::models::user;
use bcrypt::{hash, verify, DEFAULT_COST};

// Helper: hash password
pub fn hash_password(password: &str) -> String {
    hash(password, DEFAULT_COST).unwrap()
}

// Helper: verify password
pub fn verify_password(hash: &str, password: &str) -> bool {
    verify(password, hash).unwrap_or(false)
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
