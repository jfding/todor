use crate::boxops;

use actix_web::{get, web, App, HttpServer, Responder, HttpResponse};

#[get("/")]
async fn index() -> impl Responder {
    let html = include_str!("../assets/web/index.html");
    HttpResponse::Ok().body(html)
}

#[get("/boxes")]
async fn boxes() -> impl Responder {
    let (mut boxes, locked_boxes) = boxops::get_boxes();
    boxes.extend(locked_boxes.into_iter().map(|boxname| format!("[encrypted] {}", boxname)));
    HttpResponse::Ok().json(boxes)
}

#[actix_web::main]
pub async fn serve(host: &str, port: u16, secret: Option<String>) -> std::io::Result<()> {
    println!("Starting server on: http://{}:{}", host, port);
    let upgrade_path_prefix = secret.unwrap_or("v1".to_string());
    HttpServer::new(move || App::new()
        .wrap(actix_web::middleware::Logger::default())
        .service(index)
        .service(
            web::scope(&upgrade_path_prefix)
            .service(boxes))
        )
        .bind(format!("{}:{}", host, port))?
        .run()
        .await
}
