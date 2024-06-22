use actix_cors::Cors;
use actix_web::{delete, get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Serialize, Deserialize};
use rusqlite::{params, Connection};
use std::sync::Mutex;

#[derive(Serialize)]
struct GetResponse {
    data: String,
} 

#[derive(Serialize, Deserialize)]
struct Post {
    id: i32,
    title: String,
    body: String,
}

#[derive(Deserialize)]
struct NewPost {
    title: String,
    body: String,
}

#[get("/posts")]
async fn get_posts(db: web::Data<Mutex<Connection>>) -> impl Responder {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, title, body FROM posts").unwrap();
    let post_iter = stmt.query_map(params![], |row| {
        Ok(Post {
            id: row.get(0)?,
            title: row.get(1)?,
            body: row.get(2)?,
        })
    }).unwrap();

    let mut posts = Vec::new();
    for post in post_iter {
        posts.push(post.unwrap());
    }

    HttpResponse::Ok().json(posts)
}

#[post("/posts")]
async fn add_post(db: web::Data<Mutex<Connection>>, new_post: web::Json<NewPost>) -> impl Responder {
    let conn = db.lock().unwrap();

    conn.execute(
        "INSERT INTO posts (title, body) VALUES (?1, ?2)",
        &[&new_post.title, &new_post.body],
    ).unwrap();

    HttpResponse::Created().finish()
}

#[delete("/posts/{id}")]
async fn delete_post(db: web::Data<Mutex<Connection>>, post_id: web::Path<i32>) -> impl Responder {
    let conn = db.lock().unwrap();
    let result = conn.execute(
        "DELETE FROM posts WHERE id = ?1", 
        params![*post_id],
    );

    match result {
        Ok(affected_rows) => {
            if affected_rows > 0 {
                HttpResponse::Ok().finish()
            } else {
                HttpResponse::NotFound().body("Post not found")
            }
        },
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let conn = Connection::open("database.db").expect("Failed to open database");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            body TEXT NOT NULL
        )", []
    ).expect("Failed to create table");

    let conn_data = web::Data::new(Mutex::new(conn));

    HttpServer::new(move|| {
        App::new()
            .app_data(conn_data.clone())
            .wrap(
                Cors::default()
                    .allow_any_origin()
            )
            .service(get_posts)
            .service(add_post)
            .service(delete_post)
    })
    .bind(("localhost", 8000))?
    .run()
    .await
}
