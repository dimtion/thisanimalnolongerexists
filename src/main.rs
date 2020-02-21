use std::env;
use std::fs::File;

use actix_files as fs;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Result};
use askama::Template;
use dotenv::dotenv;
use rand::seq::IteratorRandom;

use wantedspecies::*;

struct AppState {
    database: Species,
    tracking_code: String,
}

#[derive(Template)]
#[template(path = "index.html")]
struct Index<'a> {
    specie: &'a Specie,
    app: &'a AppState,
}

#[derive(Template)]
#[template(path = "about.html")]
struct About<'a> {
    app: &'a AppState,
}

fn not_found() -> Result<HttpResponse> {
    // TODO: build a nice page
    Ok(HttpResponse::NotFound().body("404 NotFound"))
}

async fn about(app: web::Data<AppState>) -> Result<HttpResponse> {
    let s = About { app: &app }.render().unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

async fn index(app: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let slug: String = req.match_info().query("slug").parse().unwrap();
    let specie = match slug.len() {
        0 => match app.database.iter().choose(&mut rand::thread_rng()) {
            Some(s) => s.1,
            None => panic!(),
        },
        _ => match app.database.get(&slug) {
            Some(s) => s,
            None => return not_found(),
        },
    };
    let s = Index { app: &app, specie }.render().unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let app_db_path = env::var("APP_DB").unwrap();
    let listen = env::var("LISTEN").unwrap();

    let database: Species = {
        let db_file = File::open(app_db_path).unwrap();
        serde_yaml::from_reader(db_file).unwrap()
    };

    println!("Starting web server");
    println!("Database size: {}", &database.len());

    HttpServer::new(move || {
        let app = AppState {
            database: database.clone(),
            tracking_code: env::var("TRACKING_CODE").or::<()>(Ok(String::from(""))).unwrap(),
        };
        App::new()
            .data(app)
            .service(fs::Files::new("/static", "static/"))
            .route("/about", web::get().to(about))
            .route("/{slug:[\\w-]*}", web::get().to(index))
    })
    .bind(listen)?
    .run()
    .await
}
