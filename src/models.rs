use crate::schema::posts;

#[derive(Queryable, Clone, Debug)]
pub struct Post {
  pub id: i32,
  pub title: String,
  pub body: String,
  pub published: bool,
}

#[derive(Insertable, Clone, Debug)]
#[table_name = "posts"]
pub struct NewPost {
  pub title: String,
  pub body: String,
}
