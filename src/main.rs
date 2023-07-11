extern crate redis;
use actix_web::{web, App, HttpServer, HttpRequest, Responder, HttpResponse};
use actix_web::web::{Data, Redirect};
use tera::{Tera, Context};
use serde::{Deserialize};
use redis::Commands;
use std::sync::Mutex;


struct AppData {
    tmpl: Tera,
    con: redis::Connection,
}

#[derive(Deserialize)]
struct Content {
    contents: String,
}

async fn get_write(data: web::Data<Mutex<AppData>>, req: HttpRequest) -> impl Responder {
    let data = data.lock().unwrap();
    let key = req.match_info().get("key").unwrap();

    let mut ctx = Context::new();
    ctx.insert("key", &key.to_string());

    let rendered = data.tmpl.render("write.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

async fn post_write(data: web::Data<Mutex<AppData>>, req: HttpRequest, form: web::Form<Content>) -> impl Responder {
    let mut data = data.lock().unwrap();
    let key = req.match_info().get("key").unwrap();

    let _: () = data.con.set(key, &form.contents).unwrap();

    Redirect::to(format!("/read/{key}")).permanent()
}

async fn get_read(data: web::Data<Mutex<AppData>>, req: HttpRequest) -> impl Responder {
    let mut data = data.lock().unwrap();
    let key = req.match_info().get("key").unwrap();

    let contents: String = data.con.get(key).unwrap();

    HttpResponse::Ok().body(contents)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let tera =
            Tera::new(
                concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")
            ).unwrap();

        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let con = client.get_connection().unwrap();

        let data = Data::new(Mutex::new(AppData {tmpl: tera, con: con}));

        App::new()
            .app_data(Data::clone(&data))
            .service(
                web::resource("/write/{key}")
                    .route(web::get().to(get_write))
                    .route(web::post().to(post_write))
            )
            .service(
                web::resource("/read/{key}")
                    .route(web::get().to(get_read))
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
