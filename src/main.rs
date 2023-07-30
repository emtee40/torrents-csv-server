extern crate actix_files;
extern crate actix_web;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate rusqlite;
use actix_web::{
  middleware,
  web::{self, Data},
  App,
  HttpResponse,
  HttpServer,
};
use anyhow::{anyhow, Result};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite_pool::SqliteConnectionManager;
use rusqlite::params;
use std::{cmp, env, io, ops::Deref};
use tokio::sync::Mutex;
use uuid::Uuid;

const DEFAULT_SIZE: usize = 25;

type Conn = PooledConnection<SqliteConnectionManager>;

struct MyAppData {
  etag: String,
  pool: Pool<SqliteConnectionManager>,
}

#[actix_web::main]
async fn main() -> io::Result<()> {
  let manager = SqliteConnectionManager::file(torrents_db_file());
  let pool = r2d2::Pool::new(manager).unwrap();
  let my_app_data = Data::new(Mutex::new(MyAppData {
    etag: Uuid::new_v4().to_string(),
    pool,
  }));
  println!("Access me at {}", endpoint());
  std::env::set_var("RUST_LOG", "actix_web=debug");
  env_logger::init();

  HttpServer::new(move || {
    App::new()
      .app_data(Data::clone(&my_app_data))
      .wrap(middleware::Logger::default())
      .route("/service/search", web::get().to(search))
  })
  .keep_alive(None)
  .bind(endpoint())?
  .run()
  .await
}

fn torrents_db_file() -> String {
  env::var("TORRENTS_CSV_DB_FILE").unwrap_or_else(|_| "./torrents.db".to_string())
}

fn endpoint() -> String {
  env::var("TORRENTS_CSV_ENDPOINT").unwrap_or_else(|_| "0.0.0.0:8902".to_string())
}

#[derive(Deserialize)]
struct SearchQuery {
  q: String,
  page: Option<usize>,
  size: Option<usize>,
  type_: Option<String>,
}

async fn search(
  query: web::Query<SearchQuery>,
  data: Data<Mutex<MyAppData>>,
) -> Result<HttpResponse, actix_web::Error> {
  let my_app_data = data.lock().await;
  let etag = my_app_data.etag.clone();
  let conn = my_app_data.pool.get().unwrap();
  let res = search_query(query, conn)
    .map(|body| {
      HttpResponse::Ok()
        .append_header(("Access-Control-Allow-Origin", "*"))
        .append_header(("Cache-Control", "public, max-age=86400"))
        .append_header(("ETag", etag))
        .json(body)
    })
    .map_err(actix_web::error::ErrorBadRequest)?;
  Ok(res)
}

fn search_query(query: web::Query<SearchQuery>, conn: Conn) -> Result<Vec<Torrent>> {
  let q = query.q.trim();
  if q.is_empty() || q.len() < 3 || q == "2020" {
    return Err(anyhow!("{{\"error\": \"{}\"}}", "Empty query".to_string()));
  }

  let page = query.page.unwrap_or(1);
  let size = cmp::min(100, query.size.unwrap_or(DEFAULT_SIZE));
  let type_ = query.type_.as_ref().map_or("torrent", String::deref);
  let offset = size * (page - 1);

  println!("query = {q}, type = {type_}, page = {page}, size = {size}");

  torrent_search(conn, q, size, offset)
}

#[derive(Debug, Serialize, Deserialize)]
struct Torrent {
  infohash: String,
  name: String,
  size_bytes: isize,
  created_unix: u32,
  seeders: u32,
  leechers: u32,
  completed: Option<u32>,
  scraped_date: u32,
}

fn torrent_search(conn: Conn, query: &str, size: usize, offset: usize) -> Result<Vec<Torrent>> {
  let q = query.to_owned();
  let stmt_str = "select * from torrent where name like '%' || ?1 || '%' limit ?2, ?3";
  let mut stmt = conn.prepare(stmt_str)?;
  let torrents = stmt
    .query_map(
      params![q.replace(' ', "%"), offset.to_string(), size.to_string(),],
      |row| {
        Ok(Torrent {
          infohash: row.get(0)?,
          name: row.get(1)?,
          size_bytes: row.get(2)?,
          created_unix: row.get(3)?,
          seeders: row.get(4)?,
          leechers: row.get(5)?,
          completed: row.get(6)?,
          scraped_date: row.get(7)?,
        })
      },
    )?
    .collect::<Result<Vec<Torrent>, rusqlite::Error>>()?;

  Ok(torrents)
}
