use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::fs::File;

use std::sync::*;
use tokio::sync::Mutex;

#[derive(Deserialize)]
struct ProblemNewRequest {
    path: String,
}

#[derive(Deserialize, Default)]
struct Problem {
    id: i32,
    path: String,
}

#[derive(Serialize)]
struct PostProblemsResponse {
    id: i32,
}

#[post("/problems")]
async fn post_problems_handler(
    req: web::Json<ProblemNewRequest>,
    pool_data: web::Data<Arc<Mutex<sqlx::Pool<sqlx::MySql>>>>,
) -> impl Responder {
    let pool = pool_data.lock().await;
    let count = sqlx::query!(
        r#"SELECT COUNT(*) as value FROM problems WHERE path=? LIMIT 1;"#,
        req.path
    )
    .fetch_one(&*pool)
    .await
    .unwrap()
    .value;

    if count > 0 {
        return HttpResponse::Conflict().body(format!(
            "problem {} already registered",
            req.path.to_owned()
        ));
    }

    let path = std::env::var("PROBLEMS_ROOT")
        .expect("PROBLEMS_ROOT not set")
        .replace("\r", "")
        + &req.path
        + "/problem.yaml";
    println!("{:?}", path);
    match File::open(path) {
        Ok(_file) => true,
        Err(e) => {
            return HttpResponse::BadRequest().body(format!(
                "upload to problems repository before register. err: {}",
                e
            ));
        }
    };

    let problem_id = sqlx::query!("INSERT INTO problems (path) VALUES (?);", req.path)
        .execute(&*pool)
        .await
        .unwrap()
        .last_insert_id() as i32;

    HttpResponse::Created().json(PostProblemsResponse { id: problem_id })
}
