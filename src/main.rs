use clap::{Arg, Command};

fn main() {
    let matches = Command::new("bwenv")
        .about("TODO")
        .version("TODO")
        .arg_required_else_help(true)
        .author("nyarthan")
        .arg(
            Arg::new("token")
                .long("token")
                .short('t')
                .env("BWS_ACCESS_TOKEN"),
        )
        .get_matches();

    if let Some(token) = matches.get_one::<String>("token") {
        println!("Token: {}", token);
    }
}
