use sea_orm::DatabaseConnection;
use actix_web::{web, HttpResponse, Result};
use actix_web::Error as ActixError;
use crate::models::user::ActiveModel;
use sea_orm::{Set, ActiveModelTrait};
use bcrypt;
use crate::LoginForm;

// Signup form (GET)
pub async fn signup_form() -> HttpResponse {
    let html = r#"
        <h1>Sign Up</h1>
        <form action="/signup" method="post">
            <input type="text" name="username" placeholder="Username" required><br>
            <input type="password" name="password" placeholder="Password" required><br>
            <button type="submit">Sign Up</button>
        </form>
        <a href="/">Back</a>
    "#;
    HttpResponse::Ok().content_type("text/html").body(html)
}

// Signup handler (POST)
pub async fn signup(
    db: web::Data<DatabaseConnection>,
    form: web::Form<LoginForm>,
) -> Result<HttpResponse, ActixError> {
    let password_hash = bcrypt::hash(&form.password, bcrypt::DEFAULT_COST).unwrap();
    let user = ActiveModel {
        username: Set(form.username.clone()),
        password_hash: Set(password_hash),
        access_level: Set("None".to_string()),
        ..Default::default()
    };
    match user.insert(db.get_ref()).await {
        Ok(_) => Ok(HttpResponse::Found().append_header(("Location", "/login")).finish()),
        Err(e) => Ok(HttpResponse::InternalServerError().body(format!("Error: {}", e))),
    }
}

pub fn signup_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/signup")
            .route(web::get().to(signup_form))
            .route(web::post().to(signup))
    );
}