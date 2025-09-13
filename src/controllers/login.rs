use actix_web::HttpResponse;

// Serve login form (GET)
pub async fn login_form() -> HttpResponse {
    let html = r#"
        <h1>Login</h1>
        <form action="/login" method="post">
            <input type="text" name="username" placeholder="Username" required><br>
            <input type="password" name="password" placeholder="Password" required><br>
            <button type="submit">Login</button>
        </form>
        <a href="/">Back</a>
    "#;
    HttpResponse::Ok().content_type("text/html").body(html)
}

