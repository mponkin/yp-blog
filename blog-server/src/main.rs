use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Logger, web};
use clap::Parser;
use tonic::include_proto;
use tracing::{info, trace};

use crate::{
    application::{auth_service::AuthService, blog_service::BlogService},
    data::{post_repository::PostRepository, user_repository::UserRepository},
    domain::error::AppError,
    infrastructure::{
        database::{init_db_connection, run_migrations},
        jwt::JwtService,
        logging::init_logging,
    },
    presentation::{
        http_handlers::{
            create_post, delete_post, get_post, get_posts, login, register, update_post,
        },
        middleware::jwt_validator,
    },
};
use actix_web_httpauth::middleware::HttpAuthentication;

mod application;
mod data;
mod domain;
mod infrastructure;
mod presentation;

include_proto!("blog");

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)] // Опциональные метаданные
struct Args {
    #[arg(short = 'p', long = "port", default_value_t = 3000)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let args = Args::parse();
    dotenvy::dotenv()?;
    init_logging();
    info!("Starting blog server...");

    let url = std::env::var("DATABASE_URL")?;
    let jwt_secret = std::env::var("JWT_SECRET")?;

    let db_pool = init_db_connection(&url).await?;
    run_migrations(&db_pool).await?;

    let db_pool = Arc::new(db_pool);

    let user_repo = UserRepository::new(db_pool.clone());
    let post_repo = PostRepository::new(db_pool.clone());

    let jwt_service = Arc::new(JwtService::new(&jwt_secret));
    let auth_service = Arc::new(AuthService::new(user_repo, jwt_service.clone()));
    let blog_service = Arc::new(BlogService::new(post_repo));

    let host = "0.0.0.0";
    trace!("Starting HTTP server on {host}:{}", args.port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .max_age(3600);

        App::new().service(
            web::scope("/api")
                .wrap(Logger::default())
                .wrap(cors)
                .app_data(web::Data::new(jwt_service.clone()))
                .service(
                    web::scope("/auth")
                        .app_data(web::Data::new(auth_service.clone()))
                        .route("/register", web::post().to(register))
                        .route("/login", web::post().to(login)),
                )
                .service(
                    web::scope("/posts")
                        .app_data(web::Data::new(blog_service.clone()))
                        // no auth
                        .service(
                            web::scope("")
                                .route("/{id}", web::get().to(get_post))
                                .route("", web::get().to(get_posts)),
                        )
                        // auth required
                        .service(
                            web::scope("")
                                .wrap(HttpAuthentication::bearer(jwt_validator))
                                .route("", web::post().to(create_post))
                                .route("/{id}", web::put().to(update_post))
                                .route("/{id}", web::delete().to(delete_post)),
                        ),
                ),
        )
    })
    .bind((host, args.port))?
    .run()
    .await?;

    Ok(())
}
