#[macro_use]
extern crate diesel;
extern crate futures;
extern crate hyper;
extern crate r2d2;
extern crate tokio;
extern crate tokio_threadpool;

use futures::{future, future::poll_fn, Async, Poll};

use hyper::rt::{Future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use diesel::prelude::*;

pub mod db_pool;
pub mod models;
pub mod schema;

use self::db_pool::*;

static NOTFOUND: &[u8] = b"Not Found";
type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

// struct ReadPost {
//   conn: PgConnection,
// }

// impl Future for ReadPost {
//   type Item = String;
//   type Error = ();

//   fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
//     // self.conn.execute(query: &str, params: &[&ToSql])
//     tokio::spawn(poll_fn(move || {
//       tokio_threadpool::blocking(|| {
//         // execute read from sql
//       })
//       .unwrap();
//       Ok(Async::Ready(()))
//     }));

//     Ok(Async::Ready("I am testing hello world".to_string()))
//   }
// }

struct HelloWorld;

impl Future for HelloWorld {
  type Item = String;
  type Error = ();

  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    println!("Has been pooled");
    Ok(Async::Ready("I am testing hello world".to_string()))
  }
}

fn http_handler(req: Request<Body>, db_pool: &Pool) -> BoxFut {
  let mut response = Response::new(Body::empty());
  let db_connection = get_conn(db_pool);

  println!("Has been connecet");
  match (req.method(), req.uri().path()) {
    (&Method::GET, "/") => {
      let fut = HelloWorld {};

      let reversed = fut
        .and_then(move |hello_world| {
          *response.body_mut() = Body::from(hello_world);
          Ok(response)
        }).or_else(|_| {
          Ok(
            Response::builder()
              .status(StatusCode::NOT_FOUND)
              .body(NOTFOUND.into())
              .unwrap(),
          )
        });

      return Box::new(reversed);
    }
    _ => {
      *response.status_mut() = StatusCode::NOT_FOUND;
    }
  }

  return Box::new(future::ok(response));
}

fn main() {
  let addr = ([127, 0, 0, 1], 3000).into();
  let db_poll = init_pool();

  let new_service = move || {
    let db_poll = db_poll.clone();
    service_fn(move |req| http_handler(req, &db_poll))
  };

  let server = Server::bind(&addr)
    .serve(new_service)
    .map_err(|e| eprintln!("server error: {}", e));

  println!("Listening on http://{}", addr);
  hyper::rt::run(server);
}
