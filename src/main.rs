use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_rt::{self, Arbiter};
use actix_web::cookie::SameSite;
use actix_web::web::Data;
use actix_web::{middleware, App, HttpServer};
use rand::Rng;
use std::env;
use std::sync::*;
use tokio::sync::Mutex;

mod languages;
mod routes;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
    let address = match env::var("QKJUDGE_ADDRESS") {
        Ok(val) => val,
        Err(_e) => "localhost".to_string(),
    };
    let database = env::var("MARIADB_DATABASE").expect("MARIADB_DATABASE is not set");
    let user = env::var("MARIADB_USERNAME").expect("MARIADB_USERNAME is not set");
    let password = env::var("MARIADB_PASSWORD").expect("MARIADB_PASSWORD is not set");
    let port = env::var("DB_PORT").unwrap_or("3306".to_string());
    let host = env::var("MARIADB_HOSTNAME").unwrap_or("localhost".to_string());

    // mysql://user:pass@127.0.0.1:3306/db_name
    let database_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        user, password, host, port, database
    );

    let pool = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();

    let pool_data = Arc::new(Mutex::new(pool));

    let arbiter = Arbiter::new();
    let arbiter_data = Arc::new(Mutex::new(arbiter));
    let private_key = rand::thread_rng().gen::<[u8; 32]>();
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool_data.clone()))
            .app_data(Data::new(arbiter_data.clone()))
            .wrap(
                middleware::DefaultHeaders::new()
                    .add(("Access-Control-Allow-Origin", "https://judge.tqk.blue"))
                    .add(("Access-Control-Allow-Credentials", "true"))
                    .add((
                        "Access-Control-Allow-Methods",
                        "GET, POST, DELETE, PUT, OPTIONS",
                    ))
                    .add(("Access-Control-Allow-Headers", "Content-Type")),
            )
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&private_key)
                    .name("auth")
                    .same_site(SameSite::None)
                    .secure(true),
            ))
            .wrap(middleware::Logger::default())
            .service(routes::options_0_handler)
            .service(routes::options_1_handler)
            .service(routes::options_2_handler)
            .service(routes::get_index_handler)
            .service(routes::post_signup_handler)
            .service(routes::post_login_handler)
            .service(routes::post_logout_handler)
            .service(routes::get_ping_handler)
            .service(routes::get_hello_handler)
            .service(routes::get_whoami_handler)
            .service(routes::get_execute_handler)
            .service(routes::post_execute_handler)
            .service(routes::get_problems_handler)
            .service(routes::post_problems_handler)
            .service(routes::get_problems_pid_handler)
            .service(routes::post_submit_handler)
            .service(routes::get_submissions_sid_handler)
            .service(routes::get_submissions_handler)
            .service(routes::get_tasks_tid_handler)
            .service(routes::put_submissions_sid_handler)
            .service(routes::post_fetch_problems_handler)
    })
    .bind((address, 8080))?
    .run()
    .await
}
