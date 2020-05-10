use clap::{App, Arg, SubCommand};
use gourami_social::routes::run_server;
use gourami_social::ap;
use dotenv;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let matches = App::new("Gourami")
        .version("0.1.0")
        .author("Alex Wennerberg <alex@alexwennerberg.com>")
        .about("Gourami server and admin tools")
        .subcommand(App::new("run").about("Run server"))
        .subcommand(App::new("follow")
                    .arg(Arg::with_name("URL")
                         .help("url of the remote server to follow")
                         .required(true)
                         .index(1)
                         )
                    )
        .get_matches();
    if let Some(m) = matches.subcommand_matches("run") {
        run_server().await;
    } else if let Some(m) = matches.subcommand_matches("follow") {
            let url = m.value_of("URL").unwrap();
            ap::follow_remote_server(url).await.unwrap();
    }
        // reset password
        // follow remote
}
