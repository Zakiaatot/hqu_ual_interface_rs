use actix_web::{post, web, HttpResponse, Responder};
use crate::fake_login;
use crate::fake_login_ecard;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Info {
    stunum: String,
    password: String,
}
#[post("/")]
pub async fn index(info: web::Json<Info>) -> impl Responder {
    let res = {
        match fake_login::login(info.stunum.to_string(), info.password.to_string()).await {
            Ok(v) => {
                log::info!(
                    "{} {}: {}",
                    info.stunum.to_string(),
                    info.password.to_string(),
                    "ok!"
                );
                serde_json::json!({
                    "code":200,
                    "msg":"Ok",
                    "data":v
                })
            }
            Err(e) => {
                log::warn!(
                    "{} {}: {}",
                    info.stunum.to_string(),
                    info.password.to_string(),
                    e.to_string()
                );
                serde_json::json!({
                    "code":500,
                    "msg":"Error",
                    "data":e.to_string()
                })
            }
        }
    };
    HttpResponse::Ok()
        .content_type("application/json")
        .json(res)
}

#[post("/ecard")]
pub async fn ecard(info: web::Json<Info>) -> impl Responder {
    let res = {
        match fake_login_ecard::login(info.stunum.to_string(), info.password.to_string()).await {
            Ok(v) => {
                log::info!(
                    "{} {}: {}",
                    info.stunum.to_string(),
                    info.password.to_string(),
                    "ok!"
                );
                serde_json::json!({
                    "code":200,
                    "msg":"Ok",
                    "data":v
                })
            }
            Err(e) => {
                log::warn!(
                    "{} {}: {}",
                    info.stunum.to_string(),
                    info.password.to_string(),
                    e.to_string()
                );
                serde_json::json!({
                    "code":500,
                    "msg":"Error",
                    "data":e.to_string()
                })
            }
        }
    };
    HttpResponse::Ok()
        .content_type("application/json")
        .json(res)
}

pub async fn not_found() -> impl Responder {
    let v = serde_json::json!({
        "code":404,
        "msg":"NotFound",
        "data":""
    });
    HttpResponse::NotFound()
        .content_type("application/json")
        .json(v)
}
