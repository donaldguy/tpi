// Copyright 2023 Turing Machines
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod api;
mod auth;
mod cli;
mod legacy_handler;
mod prompt;
mod request;

use crate::legacy_handler::LegacyHandler;
use crate::auth::Auth;
use clap::{ArgMatches, CommandFactory, FromArgMatches };
use clap_complete::generate;
use cli::Cli;
use std::{io, process::ExitCode};



// ideally I'd prefer to look for something that suggests a _running_ bmcd
pub fn is_running_on_tpi_bmc() -> bool {
    std::env::consts::OS == "linux" &&
    std::fs::read_to_string(std::path::Path::new("/sys/firmware/devicetree/base/model"))
        .unwrap_or_default()
        .starts_with("Turing Pi")
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli_arg_matches = Cli::command().get_matches();


    if let Some(shell) = cli_arg_matches.get_one::<clap_complete::shells::Shell>("gen completion") {
        generate(
            shell.to_owned(),
            &mut Cli::command(),
            env!("CARGO_PKG_NAME"),
            &mut io::stdout(),
        );
        return ExitCode::SUCCESS;
    }

    if let Err(e) = execute_cli_command(cli_arg_matches).await {
        if let Some(error) = e.downcast_ref::<reqwest::Error>() {
            println!("{error}");
        } else {
            println!("{e}");
        }
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

async fn execute_cli_command(arg_matches: ArgMatches) -> anyhow::Result<()> {
    let cli = Cli::from_arg_matches(&arg_matches).unwrap();

    let command = cli.command.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "subcommand must be specified!\n\n{}",
            Cli::command().render_long_help()
        )
    })?;



    let auth =
        if cfg!(feature = "localhost") {
            Auth::Local
        } else {
            Auth::from_arg_matches(&arg_matches)
        };

    LegacyHandler::new(&cli, auth)?.handle_cmd(command).await
}
