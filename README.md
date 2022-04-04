# actix-web-isucon-measured
A middleware to measure request processing time for ISUCON.


## Usage

```rust
use actix_web_isucon_measured::{Measured, SortOptions};

use actix_web::middleware::Logger;
use actix_web::{get, web, App, HttpServer, Responder};
use std::time::Duration;

#[get("hello/{id}/{name}")]
async fn hello(params: web::Path<(u32, String)>) -> impl Responder {
    let (id, name) = params.into_inner();
    actix_web::rt::time::sleep(Duration::from_millis(rand::random::<u8>() as u64)).await;
    format!("Hello {}!    id: {}\n", name, id)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    let measured = Measured::default();

    HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(measured.clone()))
        .wrap(Logger::default())
        .wrap(measured.clone())
        .service(hello)
        .service(web::resource("/measured_tsv").route(web::get().to(|measured: web::Data<Measured>| async move {
            measured.tsv(SortOptions::SUM)
        })))
        .service(web::resource("/measured_reset").route(web::get().to(|measured: web::Data<Measured>| async move {
            measured.clear();
            "Reset OK!.\n"
        })))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await

}
```

## IUSCON11予選で練習するための詳細はこちらのブログを参照してください。

https://sengine.xyz/2022/04/04/rust3/



