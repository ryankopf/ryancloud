use actix_web::{HttpRequest, HttpResponse, Responder};

pub async fn redirect_to_https(req: HttpRequest) -> impl Responder {
    let host = req.connection_info().host().to_string();
    let uri = format!("https://{}{}", host, req.path());
    HttpResponse::PermanentRedirect().append_header(("Location", uri)).finish()
}
