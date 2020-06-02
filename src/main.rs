use clap::{App, Arg, SubCommand};
use dotenv;
use gourami_social::ap;
use gourami_social::routes::run_server;
use gourami_social::POOL;

#[macro_use]
extern crate diesel_migrations;

embed_migrations!("./migrations");

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();
    let conn = &POOL.get().unwrap();
    embedded_migrations::run(conn).unwrap();
    let matches = App::new("Gourami")
        .version("0.1.2")
        .author("Alex Wennerberg <alex@alexwennerberg.com>")
        .about("Gourami server and admin tools")
        .subcommand(App::new("run").about("Run server"))
        .subcommand(App::new("follow")
                    .arg(Arg::with_name("URL")
                         .help("url of the AP actor to follow")
                         .required(true)
                         .index(1)
                         )
                    )
        .subcommand(App::new("whitelist")
         .arg(Arg::with_name("URL")
              .help("url of the remote AP actor to whitelist. gourami will reject follows except from whitelisted AP actors. Following a remote actor also automatically whitelists that server.")
              .required(true)
              .index(1)
              )
         )

        .get_matches();
    if let Some(_) = matches.subcommand_matches("run") {
        run_server().await;
    } else if let Some(m) = matches.subcommand_matches("follow") {
        let url = m.value_of("URL").unwrap();
        ap::whitelist_or_follow_remote_server(url, true)
            .await
            .unwrap();
    } else if let Some(m) = matches.subcommand_matches("whitelist") {
        let url = m.value_of("URL").unwrap();
        ap::whitelist_or_follow_remote_server(url, false)
            .await
            .unwrap();
    }
    // reset password
    // follow remote
}
