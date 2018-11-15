use std::ops::Deref;

use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub struct DbConn(pub r2d2::PooledConnection<ConnectionManager<PgConnection>>);

impl Deref for DbConn {
  type Target = PgConnection;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

pub fn init_pool() -> Pool {
  let db = "postgres://pg:123@209.97.163.93:5432/pgdb";
  let manager = ConnectionManager::<PgConnection>::new(db);
  Pool::new(manager).expect("db pool")
}

pub fn get_conn(pool: &Pool) -> Option<DbConn> {
  match pool.get() {
    Ok(conn) => Some(DbConn(conn)),
    Err(_) => None,
  }
}
