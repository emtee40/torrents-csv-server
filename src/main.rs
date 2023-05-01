extern crate actix_files;
extern crate actix_web;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate rusqlite;

use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use anyhow::{anyhow, Result};
use rusqlite::params;
use std::{cmp, env, io, ops::Deref, path::Path};
use tokio_rusqlite::Connection;

const DEFAULT_SIZE: usize = 25;

#[actix_web::main]
async fn main() -> io::Result<()> {
  println!("Access me at {}", endpoint());
  std::env::set_var("RUST_LOG", "actix_web=debug");
  env_logger::init();

  HttpServer::new(move || {
    App::new()
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

async fn search(query: web::Query<SearchQuery>) -> Result<HttpResponse, actix_web::Error> {
  let conn = Connection::open(Path::new(&torrents_db_file()))
    .await
    .map_err(actix_web::error::ErrorBadRequest)?;
  let res = search_query(query, conn)
    .await
    .map(|body| {
      HttpResponse::Ok()
        .append_header(("Access-Control-Allow-Origin", "*"))
        .json(body)
    })
    .map_err(actix_web::error::ErrorBadRequest)?;
  Ok(res)
}

async fn search_query(query: web::Query<SearchQuery>, conn: Connection) -> Result<Vec<Torrent>> {
  let q = query.q.trim();
  if q.is_empty() || q.len() < 3 || q == "2020" {
    return Err(anyhow!("{{\"error\": \"{}\"}}", "Empty query".to_string()));
  }

  let page = query.page.unwrap_or(1);
  let size = cmp::min(100, query.size.unwrap_or(DEFAULT_SIZE));
  let type_ = query.type_.as_ref().map_or("torrent", String::deref);
  let offset = size * (page - 1);

  println!("query = {q}, type = {type_}, page = {page}, size = {size}");

  torrent_search(conn, q, size, offset).await
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

async fn torrent_search(
  conn: Connection,
  query: &str,
  size: usize,
  offset: usize,
) -> Result<Vec<Torrent>> {
  let q = query.to_owned();
  let res = conn
    .call(move |conn| {
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

      Ok::<_, rusqlite::Error>(torrents)
    })
    .await?;

  Ok(res)
}

#[cfg(test)]
mod tests {
  use crate::{torrent_search, torrents_db_file};
  use std::path::Path;
  use tokio_rusqlite::Connection;

  // #[tokio::test]
  // async fn test() {
  //   let conn = Connection::open(Path::new(&torrents_db_file()))
  //     .await
  //     .unwrap();
  //   let results = torrent_search(conn, "sherlock", 10, 0).await.unwrap();
  //   assert!(results.len() > 2);
  // }
}
