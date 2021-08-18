//! punktf - A cross-platform multi-target dotfiles manager
//!
//! ## Yet another dotfile manager?!
//!
//! Well, yes, but hear me out: This project was driven by the personal need of having to manage several dotfiles for different machines/targets. You want the same experience everywhere: On your Windows workstation along with an Ubuntu WSL instance, your Debian server and your private Arch installation. This tool fixes that problem while being cross-platform and blazingly fast. You won't need multiple sets of dotfile configurations ever again!
//!
//! Features:
//!
//! - Compile and deploy your dotfiles with one command across different platforms
//! - Use handlebar-like instructions to insert variables and compile sections conditionally
//! - Define pre- and post-hooks to customize the behavior with your own commands
//! - Create multiple profiles for different targets
//! - Works on Windows and Linux
//!
//! ## Usage
//!
//! ### Commands
//!
//! To deploy a profile, use the `deploy` subcommand:
//!
//! ```sh
//! # deploy 'windows' profile
//! punktf deploy windows
//!
//! # deploy (custom source folder)
//! punktf --source /home/demo/mydotfiles deploy windows
//! ```
//!
//! Adding the `-h`/`--help` flag to a given subcommand, will print usage instructions.
//!
//! ### Source Folder
//!
//! The punktf source folder, is the folder containing the dotfiles and punktf profiles. We recommend setting the `PUNKTF_SOURCE` environment variable, so that the dotfiles can be compiled using `punktf deploy <profile>`.
//!
//! punktf searches for the source folder in the following order:
//!
//! 1. CLI argument given with `-s`/`--source`
//! 2. Environment variable `PUNKTF_SOURCE`
//! 3. Current working directory of the shell
//!
//! The source folder should contain two sub-folders:
//!
//! * `profiles\`: Contains the punktf profile definitions (`.yaml` or `.json`)
//! * `dotfiles\`: Contains folders and the actual dotfiles
//!
//! Example punktf source folder structure:
//!
//! ```ls
//! + profiles
//!     + windows.yaml
//!     + base.yaml
//!     + arch.json
//! + dotfiles
//!     + .gitconfig
//!     + init.vim.win
//!     + base
//!         + demo.txt
//!     + linux
//!         + .bashrc
//!     + windows
//!         + alacritty.yml
//! ```
//!
//! ### Target
//!
//! Determines where `punktf` will deploy files too.
//! It can be set with:
//!
//! 1. Variable `target` in the punktf profile file
//! 2. Environment variable `PUNKTF_TARGET`
//!
//! ### Profiles
//!
//! Profiles define which dotfiles should be used. They can be a `.json` or `.yaml` file.
//!
//! Example punktf profile:
//!
//! ```yaml
//! variables:
//!   OS: "windows"
//!
//! target: "C:\\Users\\Demo"
//!
//! dotfiles:
//!   - path: "base"
//!   - path: "windows/alacritty.yml"
//!     target:
//!         Path: "C:\\Users\\Demo\\AppData\\Local\\alacritty.yml"
//!     merge: Ask
//! ```
//!
//! All properties are explained [in the wiki](https://github.com/Shemnei/punktf/wiki/Profiles).
//!
//! ## Templates
//!
//! Please refer to the [wiki](https://github.com/Shemnei/punktf/wiki/Templating) for the templating syntax.
//!
//! ## Dotfile Repositories using punktf
//!
//! - [michidk/dotfiles](https://gitlab.com/michidk/dotfiles)

mod opt;
mod util;

use clap::Clap;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use punktf_lib::deploy::executor::{Executor, ExecutorOptions};
use punktf_lib::profile::{resolve_profile, LayeredProfile, SimpleProfile};
use punktf_lib::PunktfSource;

/// Name of the environment variable which defines the default source path for
/// `punktf`.
pub const PUNKTF_SOURCE_ENVVAR: &str = "PUNKTF_SOURCE";

/// Name of the environment variable which defines the default target path for
/// `punktf`.
pub const PUNKTF_TARGET_ENVVAR: &str = "PUNKTF_TARGET";

/// Name of the environment variable which defines the default profile for
/// `punktf`.
pub const PUNKTF_PROFILE_ENVVAR: &str = "PUNKTF_PROFILE";

/// Entry point for `punktf`.
fn main() -> Result<()> {
	let _ = color_eyre::install()?;

	let opts = opt::Opts::parse();

	let log_level = match opts.shared.verbose {
		0 => log::Level::Info,
		1 => log::Level::Debug,
		_ => log::Level::Trace,
	};

	let _ = env_logger::Builder::from_env(
		env_logger::Env::default().default_filter_or(log_level.as_str()),
	)
	.init();

	log::debug!("Parsed Opts:\n{:#?}", opts);

	handle_commands(opts)
}

/// Gets the parsed command line arguments and evaluates them.
fn handle_commands(opts: opt::Opts) -> Result<()> {
	let opt::Opts {
		shared: opt::Shared { source, .. },
		command,
	} = opts;

	match command {
		opt::Command::Deploy(opt::Deploy {
			profile: profile_name,
			target,
			dry_run,
		}) => {
			let ptf_src = PunktfSource::from_root(source.into())?;

			let mut builder = LayeredProfile::build();

			// Add target cli argument to top
			let target_cli_profile = SimpleProfile {
				target,
				..Default::default()
			};
			builder.add(String::from("target_cli_argument"), target_cli_profile);

			resolve_profile(
				&mut builder,
				&ptf_src,
				&profile_name,
				&mut Default::default(),
			)?;

			// Add target environment variable to bottom
			let target_env_profile = SimpleProfile {
				target: Some(util::get_target_path()),
				..Default::default()
			};
			builder.add(
				String::from("target_environment_variable"),
				target_env_profile,
			);

			let profile = builder.finish();

			log::debug!("Profile:\n{:#?}", profile);
			log::debug!("Source: {}", ptf_src.root().display());
			log::debug!("Target: {:?}", profile.target_path());

			// Setup environment
			std::env::set_var("PUNKTF_CURRENT_SOURCE", ptf_src.root());
			if let Some(target) = profile.target_path() {
				std::env::set_var("PUNKTF_CURRENT_TARGET", target);
			}
			std::env::set_var("PUNKTF_CURRENT_PROFILE", profile_name);

			let options = ExecutorOptions { dry_run };

			let deployer = Executor::new(options, util::ask_user_merge);

			let deployment = deployer.deploy(ptf_src, &profile);

			match deployment {
				Ok(deployment) => {
					log::debug!("Deployment:\n{:#?}", deployment);
					util::log_deployment(&deployment);

					if deployment.status().is_failed() {
						Err(eyre!("Some dotfiles failed to deploy"))
					} else {
						Ok(())
					}
				}
				Err(err) => {
					log::error!("Deployment aborted: {}", err);
					Err(err)
				}
			}
		}
	}
}
