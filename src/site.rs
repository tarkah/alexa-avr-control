use actix_web::{guard, middleware, web, App, HttpResponse, HttpServer};
use alexa_sdk::{Request, Response};

fn index(item: web::Json<Request>, req: web::HttpRequest) -> HttpResponse {
    println!("request: {:#?}", &item.0);
    println!("intent: {:#?}", &item.intent());
    println!("slot: {:#?}", &item.slot_value("Number"));

    println!("{:#?}", req.headers());
    if item.intent() == alexa_sdk::request::IntentType::Help {
        return HttpResponse::Ok().json(Response::new(false).speech(
            alexa_sdk::response::Speech::ssml(
                "<speak><say-as interpret-as=\"interjection\">okey dokey</say-as>.</speak>",
            ),
        ));
    }
    if item.intent() == alexa_sdk::request::IntentType::Fallback {
        return HttpResponse::Ok().json(Response::end());
    }

    HttpResponse::Ok().json(Response::new(false))
}

pub fn run() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

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
