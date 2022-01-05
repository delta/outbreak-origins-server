use actix_cors::Cors;
use actix_web::http;

pub fn cors_config() -> Cors {
    Cors::default()
        .allowed_origin("http://localhost:8001")
        .allowed_methods(vec!["POST"])
        .allowed_headers(vec![
            http::header::CONTENT_TYPE,
            http::header::ACCESS_CONTROL_ALLOW_HEADERS,
            http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
            http::header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
        ])
        .supports_credentials()
}
