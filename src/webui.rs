use crate::boxops;
use crate::taskbox;

use actix_web::{get, post, web, App, HttpServer, Responder, HttpResponse};
use serde_json::json;

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

#[get("/boxes/{boxname}")]
async fn box_contents(boxname: web::Path<String>) -> impl Responder {
    if let Some(mut tb) = taskbox::TaskBox::from_boxname(&boxname.into_inner()) {
        HttpResponse::Ok().json(json!({
            "todos": tb.get_all_todos()
        }))
    } else {
        HttpResponse::NotFound().body("Taskbox not found")
    }
}

#[post("/boxes/{boxname}")]
async fn update_box(boxname: web::Path<String>, body: String) -> impl Responder {
    if let Some(mut tb) = taskbox::TaskBox::from_boxname(&boxname.into_inner()) {
        tb.add(body, None, false, "now");

        HttpResponse::Ok().body("Taskbox updated")
    } else {
        HttpResponse::NotFound().body("Taskbox not found")
    }
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
            .service(boxes)
            .service(box_contents)
            .service(update_box))
        )
        .bind(format!("{}:{}", host, port))?
        .run()
        .await
}
