mod api;
mod captcha_solver;
mod crypto;
mod fake_login;
mod fake_login_ecard;
use actix_web::{web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    log::info!("Server start on: http://0.0.0.0:8085");
    HttpServer::new(|| {
        App::new()
            .service(api::index)
            .service(api::ecard)
            .default_service(web::to(api::not_found))
    })
    .bind(("0.0.0.0", 8085))?
    .run()
    .await
}
