use std::{net::SocketAddr, sync::Arc};

use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Logger, web};
use blog_grpc_api::blog_service_server::BlogServiceServer;
use clap::Parser;

use tokio::{signal, sync::oneshot::Receiver};
use tracing::{error, info, trace, warn};

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
        grpc_service::GrpcService,
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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long = "http_port", default_value_t = 8080)]
    http_port: u16,
    #[arg(long = "grpc_port", default_value_t = 50051)]
    grpc_port: u16,
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

    let (mut http_task, http_server_handle) = {
        let http_port = args.http_port;
        let jwt_service = jwt_service.clone();
        let auth_service = auth_service.clone();
        let blog_service = blog_service.clone();

        let http_server =
            setup_http_server(host, http_port, jwt_service, auth_service, blog_service)?;

        let http_server_handle = http_server.handle();

        let http_task_handle = tokio::spawn(http_server);

        (http_task_handle, http_server_handle)
    };

    let (grpc_shutdown_tx, grpc_shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let mut grpc_task = {
        let grpc_port = args.grpc_port;
        let jwt_service = jwt_service.clone();
        let auth_service = auth_service.clone();
        let blog_service = blog_service.clone();

        tokio::spawn(async move {
            run_grpc_server(
                host,
                grpc_port,
                jwt_service,
                auth_service,
                blog_service,
                grpc_shutdown_rx,
            )
            .await
        })
    };

    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Ctrl+C received. Shutting down...");

            let _ = grpc_shutdown_tx.send(());

            let _ = http_server_handle.stop(true).await;

            let (http_res, grpc_res) = tokio::join!(http_task, grpc_task);

            if let Err(e) = http_res {
                warn!("HTTP task finished with error: {e}");
            }

            if let Err(e) = grpc_res {
                warn!("GRPC task finished with error: {e}");
            }
        },
        res = &mut http_task => {
            error!("HTTP server stopped unexpectedly: {:?}", res);
        },
        res = &mut grpc_task => {
            error!("GRPC server stopped unexpectedly: {:?}", res);
        }
    }

    info!("Blog server shut down");

    Ok(())
}

fn setup_http_server(
    host: &str,
    port: u16,
    jwt_service: Arc<JwtService>,
    auth_service: Arc<AuthService>,
    blog_service: Arc<BlogService>,
) -> Result<actix_web::dev::Server, AppError> {
    trace!("Starting HTTP server on {host}:{}", port);
    let auth_service = web::Data::new(auth_service);
    let blog_service = web::Data::new(blog_service);
    let jwt_service = web::Data::new(jwt_service);

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .max_age(3600);

        App::new().app_data(jwt_service.clone()).service(
            web::scope("/api")
                .wrap(cors)
                .wrap(Logger::default())
                .service(
                    web::scope("/auth")
                        .app_data(auth_service.clone())
                        .route("/register", web::post().to(register))
                        .route("/login", web::post().to(login)),
                )
                .service(
                    web::scope("/posts")
                        .app_data(blog_service.clone())
                        .route("", web::get().to(get_posts))
                        .service(
                            web::resource("")
                                .wrap(HttpAuthentication::bearer(jwt_validator))
                                .route(web::post().to(create_post)),
                        )
                        .service(
                            web::scope("/{id}")
                                .route("", web::get().to(get_post))
                                .service(
                                    web::resource("")
                                        .wrap(HttpAuthentication::bearer(jwt_validator))
                                        .route(web::put().to(update_post))
                                        .route(web::delete().to(delete_post)),
                                ),
                        ),
                ),
        )
    })
    .bind((host, port))?;

    Ok(server.run())
}

async fn run_grpc_server(
    host: &str,
    port: u16,
    jwt_service: Arc<JwtService>,
    auth_service: Arc<AuthService>,
    blog_service: Arc<BlogService>,
    grpc_shutdown_rx: Receiver<()>,
) -> Result<(), AppError> {
    let grpc_service =
        BlogServiceServer::new(GrpcService::new(auth_service, blog_service, jwt_service));

    let grpc_address: SocketAddr = format!("{host}:{}", port).parse()?;

    trace!("Starting GRPC server on {}", grpc_address);

    tonic::transport::Server::builder()
        .add_service(grpc_service)
        .serve_with_shutdown(grpc_address, async {
            let _ = grpc_shutdown_rx.await;
            trace!("GRPC received shutdown signal")
        })
        .await?;

    Ok(())
}
