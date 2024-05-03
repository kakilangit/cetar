#![allow(dead_code)]
use cetar::{make_color, print_error};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "cetar", about = "💥 CURL execution timing analyzer", version, long_about = None)]
struct Args {
    url: String,

    #[clap(
        short = 'X',
        long,
        default_value = "GET",
        help = "Available methods: GET, HEAD, POST, PUT, DELETE, CONNECT, OPTIONS, TRACE, PATCH"
    )]
    method: String,

    #[clap(
        short = 'H',
        long,
        help = "Pass custom header(s) to server, example: -H 'Accept: application/json'"
    )]
    headers: Vec<cetar::network::Header>,

    #[clap(
        short,
        long,
        help = "HTTP request data to send, example: -d 'key=value' -d @file.json -d '{\"key\": \"value\"}'"
    )]
    data: Option<String>,

    #[clap(short, long, help = "Write output to <file>")]
    output: Option<String>,

    #[clap(short = 'l', long = "location", help = "Follow HTTP 3xx redirects")]
    follow_redirects: bool,

    #[clap(short, long, help = "Verbose output")]
    verbose: bool,

    #[clap(short = 'B', long, help = "Display response body")]
    display_response_body: bool,

    #[clap(short = 'G', long, help = "Display response headers")]
    display_response_headers: bool,

    #[clap(
        long,
        default_value = "cyan",
        help = "Main output color, available colors: black, red, green, yellow, blue, magenta, cyan, white"
    )]
    color: String,
}

impl TryFrom<Args> for cetar::network::Config<'_> {
    type Error = anyhow::Error;

    fn try_from(cli: Args) -> Result<Self, Self::Error> {
        let data = if let Some(d) = cli.data {
            match d.starts_with('@') {
                true => std::fs::read_to_string(&d[1..])?,
                false => d,
            }
            .into()
        } else {
            None
        };

        Ok(Self {
            url: cli.url.into(),
            request_headers: cli.headers,
            request_body: data.map(|x| x.into()),
            method: cetar::network::Method::try_from(cli.method)?,
            color: cetar::color::Color::try_from(cli.color)?,
            output: cli.output.map(|x| x.into()),
            display_response_body: cli.display_response_body,
            display_response_headers: cli.display_response_headers,
            follow_redirects: cli.follow_redirects,
            verbose: cli.verbose,
        })
    }
}

fn execute() -> anyhow::Result<()> {
    let parsed = Args::parse();
    let config = cetar::network::Config::try_from(parsed)?;
    let result = cetar::network::send_request(&config)?;

    cetar::output::handle_output(&config, &result)?;
    cetar::output::Screen::new(&config, &result).display();

    Ok(())
}

fn main() {
    if let Err(e) = execute() {
        print_error!("Error: {}", e);
    }
}
