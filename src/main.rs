pub mod handlers;

use actix_web::{App, HttpServer, middleware, web};
use clap::{Arg, Command};

struct AppState {

}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let matches = Command::new("visbackend")
        .arg(
            Arg::new("port")
                .short('p')
                .value_name("PORT")
                .help("Listen port"),
        )
        .arg(
            Arg::new("ip")
                .short('j')
                .value_name("IP")
                .help("Listen address"),
        )
        .arg(
            Arg::new("serve")
                .short('s')
                .value_name("SERVE")
                .help("Serve files from directory")
        )
        .get_matches();

    let listen_ip: String = matches
        .get_one::<String>("ip")
        .unwrap_or(&"127.0.0.1".to_string())
        .to_string();

    let listen_port: u16 = matches
        .get_one::<String>("port")
        .unwrap_or(&"8019".to_string())
        .parse::<u16>()
        .expect("Invalid port");
    
    let serve: Option<String> = matches
        .get_one::<String>("serve").cloned();

    log::info!(
        "starting HTTP server at http://{}:{}",
        listen_ip,
        listen_port
    );

    let state = web::Data::new(AppState {

    });

    HttpServer::new(move || {
        let mut app = App::new()
            .app_data(state.clone())
            .service(crate::handlers::echo)
            ;
            match &serve {
                Some(serve_path) => {
                    app = app.service(actix_files::Files::new("/", serve_path).index_file("index.html")) 
                },
                None => ()
            }
            app.wrap(middleware::Logger::default())
    })
    .workers(2)
    .bind((listen_ip, listen_port))?
    .run()
    .await
}
