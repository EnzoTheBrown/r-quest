extern crate diesel;

use crate::model;
use crate::schema;
use actix_web::{get, middleware, post, web, App, HttpResponse, HttpServer, Responder};
use diesel::{prelude::*, r2d2};
type DbPool = r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>;

#[get("/projects")]
async fn get_projects(pool: web::Data<DbPool>) -> impl Responder {
    let projects = schema::project::table
        .load::<model::Project>(&mut pool.get().expect("Failed to get DB connection"))
        .expect("Error loading projects");
    HttpResponse::Ok().json(projects)
}

#[get("/projects/{project_id}/requests")]
async fn get_requests(pool: web::Data<DbPool>, project_id: web::Path<i32>) -> impl Responder {
    let project_id = project_id.into_inner();
    let requests = schema::request::table
        .filter(schema::request::project_id.eq(project_id))
        .load::<model::Request>(&mut pool.get().expect("Failed to get DB connection"))
        .expect("Error loading requests");
    HttpResponse::Ok().json(requests)
}

#[post("/projects")]
async fn create_project(
    pool: web::Data<DbPool>,
    new_project: web::Json<model::NewProject>,
) -> impl Responder {
    let new_project = new_project.into_inner();

    diesel::insert_into(schema::project::table)
        .values(&new_project)
        .execute(&mut pool.get().expect("Failed to get DB connection"))
        .expect("Error inserting new project");

    HttpResponse::Created().finish()
}

#[get("/projects/{project_id}/variables")]
async fn get_variables(pool: web::Data<DbPool>, project_id: web::Path<i32>) -> impl Responder {
    let project_id = project_id.into_inner();
    let variables = schema::variable::table
        .filter(schema::variable::project_id.eq(project_id))
        .load::<model::Variable>(&mut pool.get().expect("Failed to get DB connection"))
        .expect("Error loading variables");
    HttpResponse::Ok().json(variables)
}

fn initialize_db_pool() -> DbPool {
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let manager = r2d2::ConnectionManager::<SqliteConnection>::new(conn_spec);
    r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be valid path to SQLite DB file")
}

pub async fn run_server() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let pool = initialize_db_pool();

    log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .service(get_projects)
            .service(get_requests)
            .service(get_variables)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
