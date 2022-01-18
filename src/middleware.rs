use actix_cors::Cors;
use actix_web::http;

pub fn cors_config() -> Cors {
    Cors::default()
        .allow_any_origin()
        .allowed_methods(vec!["GET", "POST"])
        .allowed_headers(vec![
            http::header::CONTENT_TYPE,
            http::header::ACCESS_CONTROL_ALLOW_HEADERS,
            http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
            http::header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
        ])
        .supports_credentials()
}
