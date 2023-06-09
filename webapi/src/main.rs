#[macro_use]
extern crate rbatis;

use std::{fs::File, fmt::Debug};
use std::io::prelude::*;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result, HttpRequest};
use log::info;
use log4rs;
use serde::{Deserialize,Serialize};
use rbatis::{rbatis::Rbatis, rbdc::datetime::DateTime};
use rbdc_sqlite::driver::SqliteDriver;
use rbdc_pg::driver::PgDriver;
use polars::prelude::*;
use chrono::prelude::*;

#[derive(Debug, Deserialize)]
struct Config {
   db1: DBInfo,
   db2: DBInfo,
}

#[derive(Debug, Deserialize)]
struct DBInfo {
    kind: String,
    connect: String,
}


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


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Option<i32>,
    pub username: Option<String>,
    pub firstname: Option<String>,
    pub lastname: Option<String>,
}
rbatis::crud!(UserInfo {},"public.user");

impl_select!(UserInfo{select_by_name(name:String) => "`where username = #{name} limit 1`"},"public.user");


#[post("/user")]
async fn add_user(repo: web::Data<Repository>,info: web::Json<UserInfo>) -> Result<String> {
    info!("add_user");

    let user = UserInfo{
        id: None,
        username: info.username.clone(),
        firstname: info.firstname.clone(),
        lastname: info.lastname.clone(),
    };

    let mut conn2 = repo.db2.clone();
    let data = UserInfo::insert(&mut conn2, &user).await.unwrap();
    info!("insert = {:?}", data);

    Ok(format!("Welcome {}!", info.username.clone().unwrap()))
}

#[get("/user/{name}")]
async fn get_user(repo: web::Data<Repository>,name: web::Path<String>,req: HttpRequest) -> Result<impl Responder> {    
    info!("get_user");

    let cont_type = req.headers().get("Content-Type").unwrap();
    info!("Content-Type: {}", cont_type.to_str().unwrap());

    let mut conn = repo.db2.clone();
    let data = UserInfo::select_by_name(&mut conn, name.to_string()).await.unwrap();

    Ok(web::Json(data[0].clone()))
}

async fn manual_hello() -> impl Responder {
    info!("manual_hello");

    HttpResponse::Ok().body("Hey there!")
}

#[derive(Clone)]
struct Repository {
    db1: Rbatis,
    db2: Rbatis,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WinMem {
    pub timestamp: Option<DateTime>,
    pub host: Option<String>,
    pub standby_cache_reserve_bytes: Option<i32>,
    pub demand_zero_faults_persec: Option<f32>,
}
rbatis::crud!(WinMem{},"public.win_mem");

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    info!("booting up");

    let mut f = File::open("config/setting.toml").unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
    let config: Config = toml::from_str(&contents).unwrap();

    let ( rb1, rb2 ) = get_connect_db( config );

    let repos = Repository {
        db1: rb1.clone(),
        db2: rb2.clone(),
    };

    let ct = rb1.exec("CREATE TABLE IF NOT EXISTS process (event text PRIMARY KEY, timestamp text, indicate_key text, indicate_value text);", vec![]).await.unwrap();
    info!("CREATE TABLE: [{}]", ct.to_string());

    let count: u64 = rb2
        .query_decode("select count(*) from public.user;", vec![])
        .await
        .unwrap();
    info!("user count={}", count);

    let table: Vec<WinMem> = rb2
        .query_decode("select timestamp, host, \"Standby_Cache_Reserve_Bytes\" as standby_cache_reserve_bytes, \"Demand_Zero_Faults_persec\" as demand_zero_faults_persec from public.win_mem limit 10;", vec![])
        .await
        .unwrap();
    info!("data table={:?}", table);
    // rbdc::types::datetime to chrono::naive::datetime
    let mut v1: Vec<String> = Vec::new();
    let mut v2: Vec<String> = Vec::new();
    let mut v3: Vec<i32> = Vec::new();
    let mut v4: Vec<f32> = Vec::new();
    for row in table {
        let dt: DateTime = row.timestamp.unwrap();
        v1.push(format!("{}-{:02}-{:02}T{:02}:{:02}:{:02}",dt.year,dt.mon,dt.day,dt.hour,dt.min,dt.sec));
        v2.push(row.host.unwrap());
        v3.push(row.standby_cache_reserve_bytes.unwrap());
        v4.push(row.demand_zero_faults_persec.unwrap());
    }
    let fmt = "%Y-%m-%dT%H:%M:%S";
    let date_series: Vec<NaiveDateTime> = v1.iter().map(|date_str| NaiveDateTime::parse_from_str(date_str.as_str(), fmt).unwrap()).collect();
    let s1 = Series::new("timestamp", date_series);
    let s2 = Series::new("host", v2);
    let s3 = Series::new("standby", v3);
    let s4 = Series::new("demand", v4);
    let df = DataFrame::new(vec![s1,s2,s3,s4]).unwrap();
    info!("df={:?}", df.shape() );
    info!("df:{:?}", df["timestamp"] );

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(repos.clone()))
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

fn get_connect_db( config: Config ) -> (Rbatis,Rbatis) {
    let rb1 = get_connect( config.db1.kind, config.db1.connect );
    let rb2 = get_connect( config.db2.kind, config.db2.connect );
    (rb1,rb2)
}

fn get_connect( drv: String, con: String ) -> Rbatis {
    let rb: Rbatis = Rbatis::new();
    if drv == "SQLite" {
        rb.init(SqliteDriver{}, con.as_str()).unwrap();
    } else if drv == "PostgreSQL" {
        rb.init(PgDriver{}, con.as_str()).unwrap();
    }
    rb
}


#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{
        test, http, web::Bytes,
    };

    #[actix_web::test]
    async fn test_manual_hello() {
        let app = test::init_service(App::new().route("/", web::get().to(manual_hello))).await;
        let req = test::TestRequest::get().to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        let result = test::read_body(resp).await;
        assert_eq!(result, Bytes::from_static(b"Hey there!"));
    }

    #[actix_web::test]
    async fn test_hello() {
        let app = test::init_service(App::new().service(hello)).await;
        let req = test::TestRequest::get().to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        let result = test::read_body(resp).await;
        assert_eq!(result, Bytes::from_static(b"Hello world!"));
    }

    #[actix_web::test]
    async fn test_echo() {
        let app = test::init_service(App::new().service(echo)).await;
        let payload = r#"test string"#.as_bytes();
        let req = test::TestRequest::post()
            .uri("/echo")
            .set_payload(payload)
            .to_request();
        let result = test::call_and_read_body(&app, req).await;
        assert_eq!(result, Bytes::from_static(b"test string"));
    }
}