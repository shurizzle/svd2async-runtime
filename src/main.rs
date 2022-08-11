mod one;
pub mod utils;
use clap::Parser;

#[allow(clippy::type_complexity)]
const VERSIONS: &[fn(&str) -> Result<(), Box<dyn std::error::Error>>] = &[one::run];

fn valid_version(s: &str) -> Result<usize, String> {
    let version: usize = s.parse().map_err(|_| format!("`{}` isn't a number", s))?;
    if (1..=VERSIONS.len()).contains(&version) {
        Ok(version - 1)
    } else {
        Err(format!("Version not in range 1..{}", VERSIONS.len()))
    }
}

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value_t = VERSIONS.len(), value_parser = valid_version)]
    version: usize,
    #[clap(multiple = false)]
    file: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    VERSIONS[args.version](&args.file)
}
