use actix_web::body::{BoxBody, MessageBody};
use actix_web::{web, App, Error, HttpServer, Responder};
use mime_guess::from_path;
use rust_embed::RustEmbed;

use actix::Actor;
use actix::StreamHandler;
use actix_web::HttpResponse;
use actix_web_actors::ws;
use std::borrow::Cow;

#[derive(RustEmbed)]
#[folder = "../frontend/static/"]
struct Asset;

fn handle_embedded_file(path: &str) -> HttpResponse {
    match Asset::get(path) {
        Some(content) => {
            let body: BoxBody = match content.data {
                Cow::Borrowed(bytes) => MessageBody::boxed(bytes),
                Cow::Owned(bytes) => MessageBody::boxed(bytes),
            };
            HttpResponse::Ok()
                .content_type(from_path(path).first_or_octet_stream().as_ref())
                .body(body)
        }
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

async fn index() -> HttpResponse {
    handle_embedded_file("html/index.html")
}

async fn dist(path: web::Path<String>) -> HttpResponse {
    handle_embedded_file(&path)
}

/// Define HTTP actor
struct MyWs;

impl Actor for MyWs {
    type Context = actix_web_actors::ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<actix_web_actors::ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        println!("Handle Stream!");
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) if text == "login u p" => ctx.text("login successful"),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

async fn websocket(
    req: actix_web::HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let resp = ws::start(MyWs {}, &req, stream);
    println!("{:?}", resp);
    resp
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/ws/", web::get().to(websocket))
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/{_:.*}").route(web::get().to(dist)))
            .service(web::resource("/dist/{_:.*}").route(web::get().to(dist)))
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
