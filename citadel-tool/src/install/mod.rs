use std::process::exit;

mod cli;
mod disk;
pub(crate) mod installer;

pub fn main(args: Vec<String>) {
    let mut args = args.iter().skip(1);
    let result = if let Some(dev) = args.next() {
        cli::run_cli_install_with(dev)
    } else {
        cli::run_cli_install()
    };

    let ok = match result {
        Ok(ok) => ok,
        Err(ref err) => {
            println!("Install failed: {}", err);
            exit(1);
        }
    };
    if !ok {
        println!("Install cancelled...");
    }
}
