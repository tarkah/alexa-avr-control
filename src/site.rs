use crate::skill::process_request;
use actix_web::{guard, middleware, web, App, HttpResponse, HttpServer};
use alexa_sdk::Request;
use log::{debug, info};

fn index(item: web::Json<Request>) -> HttpResponse {
    info!("Request received...");
    debug!("{:?}", item.0);
    let response = process_request(item.into_inner());
    info!("Sending back response...");
    debug!("{:?}", response);

    HttpResponse::Ok().json(response)
}

pub fn run() -> std::io::Result<()> {
    info!("Starting server on 0.0.0.0:8086");

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/").route(
                    web::route()
                        .guard(guard::Header(
                            "content-type",
                            "application/json; charset=utf-8",
                        ))
                        .guard(guard::Post())
                        .to(index),
                ),
            )
    })
    .bind("0.0.0.0:8086")?
    .workers(1)
    .run()
}
