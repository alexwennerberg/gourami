use clap::{App, Arg, SubCommand};
use gourami_social::routes::run_server;

#[tokio::main]
async fn main() {
    let matches = App::new("Gourami")
        .version("0.0.0")
        .author("Alex Wennerberg <alex@alexwennerberg.com>")
        .about("Gourami server and admin tools")
        .subcommand(
            App::new("run")
            .about("Run server")
            )
        .subcommand(
            App::new("admin")
            .about("Admin Tools")
            ).get_matches();
    if let Some(m) = matches.subcommand_matches("run") {
        run_server().await;
    }
    else if let Some(m) = matches.subcommand_matches("admin") {
        // write admin commands here
        // reset password
        // follow remote
    }
}
