mod cli;
mod commands;
mod scenes;

use clap::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::Cli::parse();

    match args.command {
        cli::Command::Render(cmd) => {
            commands::render::handle_render_command(cmd)?;
        }
    }

    Ok(())
}
