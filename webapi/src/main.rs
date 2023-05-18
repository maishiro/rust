use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use log::{error, info, warn};
use log4rs;
use serde::{Deserialize,Serialize};
use rbatis::rbatis::Rbatis;
use rbdc_sqlite::driver::SqliteDriver;
use rbdc_pg::driver::PgDriver;

#[get("/")]
async fn hello() -> impl Responder {
    info!("/");

    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    info!("/echo");

    HttpResponse::Ok().body(req_body)
}

#[derive(Deserialize)]
struct UserInfo {
    username: String,
    firstname: String,
    lastname: String,
}

/// deserialize `Info` from request's body
#[post("/user")]
async fn add_user(info: web::Json<UserInfo>) -> Result<String> {
    info!("add_user");

    Ok(format!("Welcome {}!", info.username))
}


#[derive(Serialize)]
struct MyObj {
    name: String,
}

#[get("/user/{name}")]
async fn get_user(rb: web::Data<Rbatis>,name: web::Path<String>) -> Result<impl Responder> {
    info!("get_user");

    let mut conn = rb.acquire().await.unwrap();
    let count: u64 = conn
        .query_decode("select count(*) from win_cpu", vec![])
        .await
        .unwrap();
    println!(">>>>> count={}", count);

    let obj = MyObj {
        name: name.to_string(),
    };
    Ok(web::Json(obj))
}

async fn manual_hello() -> impl Responder {
    info!("manual_hello");

    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    info!("booting up");

    let rb = Rbatis::new();
    // connect to database  
    // postgresql 
    rb.init(PgDriver{},"postgres://postgres:postgres@localhost:5432/telegraf").unwrap();

    let count: u64 = rb
        .query_decode("select count(*) from win_cpu", vec![])
        .await
        .unwrap();
    println!(">>>>> count={}", count);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(rb.clone()))
            .service(hello)
            .service(echo)
            .service(add_user)
            .service(get_user)
            .route("/hey", web::get().to(manual_hello))
    })
    // .bind(("127.0.0.1", 8080))?
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
