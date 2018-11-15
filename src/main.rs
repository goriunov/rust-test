#[macro_use]
extern crate diesel;
extern crate futures;
extern crate hyper;
extern crate r2d2;
extern crate tokio;
extern crate tokio_threadpool;

use futures::{future, Future, Poll};

use diesel::prelude::*;
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};

pub mod db_pool;
pub mod models;
pub mod schema;

use self::db_pool::*;
use self::models::NewPost;
use self::schema::posts;

static NOT_FOUND: &[u8] = b"Not Found";
type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

struct PutPost {
  pool: Pool,
  new_post: NewPost,
}

impl Future for PutPost {
  type Item = ();
  type Error = ();

  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    let db_connection = &*get_conn(&self.pool).unwrap();

    tokio_threadpool::blocking(|| {
      diesel::insert_into(posts::table)
        .values(&self.new_post)
        .execute(db_connection)
        .unwrap();

      // uncomment next line to speed up everything
      // println!("Print me");
    }).map_err(|e| {
      println!("Got error in thread {:?}", e);
    })
  }
}

fn http_handler(req: Request<Body>, db_pool: &Pool) -> BoxFut {
  let mut response = Response::new(Body::empty());

  match (req.method(), req.uri().path()) {
    (&Method::GET, "/") => {
      let put_post = PutPost {
        pool: db_pool.clone(),
        new_post: NewPost {
          title: String::from("Hello world"),
          body: String::from("Hope it works"),
        },
      };

      let new_post_insert = put_post
        .and_then(|_| {
          *response.body_mut() = Body::from("We should have put everything properly");
          Ok(response)
        }).or_else(|_| {
          Ok(
            Response::builder()
              .status(StatusCode::NOT_FOUND)
              .body(NOT_FOUND.into())
              .unwrap(),
          )
        });

      return Box::new(new_post_insert);
    }
    _ => {
      *response.status_mut() = StatusCode::NOT_FOUND;
    }
  }

  return Box::new(future::ok(response));
}

fn main() {
  let addr = ([0, 0, 0, 0], 3000).into();
  let db_poll = init_pool();

  let new_service = move || {
    let db_poll = db_poll.clone();
    service_fn(move |req| http_handler(req, &db_poll))
  };

  let server = Server::bind(&addr)
    .serve(new_service)
    .map_err(|e| eprintln!("server error: {}", e));

  println!("Listening on http://{}", addr);
  tokio::run(server);
}
