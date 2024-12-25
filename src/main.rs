mod config;
mod constant;

use anyhow::{anyhow, Result};
use clap::{arg, Command as ClapCommand};
use config::Config;
use constant::{LAUNCHD_PLIST_PATH, MODE_ONCE, MODE_WATCH};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::{env, process::Stdio};
use totp_rs::{Algorithm, Secret, TOTP};

fn main() -> Result<()> {
    let matches = ClapCommand::new("vpn-helper")
        .subcommand(
            ClapCommand::new("connect")
                .about("Connect to VPN")
                .arg(arg!(-m --mode [MODE] "The running mode").default_value(MODE_ONCE))
                .arg(arg!(-e --env [ENV] "The environment file directory").default_value(".env")),
        )
        .subcommand(ClapCommand::new("disconnect").about("Disconnect the VPN"))
        .subcommand(
            ClapCommand::new("add-service")
                .about("Add launchd service")
                .arg(arg!(-e --env [ENV] "The environment file directory").default_value(".env")),
        )
        .subcommand(ClapCommand::new("remove-service").about("Remove launchd service"))
        .get_matches();
    check_root()?;
    match matches.subcommand() {
        Some(("connect", sub_matches)) => connect_vpn(
            sub_matches.get_one::<String>("mode").unwrap(),
            sub_matches.get_one::<String>("env").unwrap(),
        ),
        Some(("disconnect", _)) => disconnect(),
        Some(("add-service", sub_matches)) => {
            add_launchd_service(sub_matches.get_one::<String>("env").unwrap())
        }
        Some(("remove-service", _)) => remove_launchd_service(),
        _ => Err(anyhow!(
            "Invalid command. Try 'vpn-helper --help' for more information"
        )),
    }
}

fn connect_vpn(mode: &str, env: &str) -> Result<()> {
    check_required_programs()?;
    dotenv::from_path(Path::new(env).canonicalize()?)?;
    let config = Config::from_env()?;
    let token = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        Secret::Encoded(config.totp_secret).to_bytes()?,
    )?
    .generate_current()?;
    let script = format!("vpn-slice {}", config.route_cidr);
    let mut options = vec![
        "--user",
        &config.username,
        "--passwd-on-stdin",
        "--script",
        &script,
    ];
    if mode == MODE_ONCE {
        options.push("--background");
    }
    let mut child = Command::new("openconnect")
        .args(options)
        .arg(config.host)
        .stdin(Stdio::piped())
        .spawn()?;
    write!(
        child.stdin.take().ok_or(anyhow!("Failed to open stdin"))?,
        "{}{}",
        config.password,
        token
    )?;
    println!("OpenConnect is running on {}.", child.id());
    if mode == MODE_WATCH {
        child.wait()?;
    }
    Ok(())
}

fn add_launchd_service(env: &str) -> Result<()> {
    let mut file = File::create(LAUNCHD_PLIST_PATH)?;
    write!(
        file,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>KeepAlive</key>
	<true/>
	<key>Label</key>
	<string>com.moecm.vpn-helper</string>
	<key>LimitLoadToSessionType</key>
	<array>
		<string>Aqua</string>
		<string>Background</string>
		<string>LoginWindow</string>
		<string>StandardIO</string>
		<string>System</string>
	</array>
	<key>ProgramArguments</key>
	<array>
		<string>{}</string>
		<string>connect</string>
		<string>--mode</string>
		<string>watch</string>
		<string>--env</string>
		<string>{}</string>
	</array>
	<key>RunAtLoad</key>
	<true/>
	<key>EnvironmentVariables</key>
	<dict>
		<key>PATH</key>
		<string>/opt/homebrew/bin:/opt/homebrew/sbin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin</string>
	</dict>
</dict>
</plist>"#,
        env::current_exe()?
            .to_str()
            .ok_or(anyhow!("Invalid path"))?,
        Path::new(env).canonicalize()?.display()
    )?;
    Command::new("launchctl")
        .args(["bootstrap", "system", LAUNCHD_PLIST_PATH])
        .status()?;
    println!("VPN Helper has been added to launchd.");
    Ok(())
}

fn remove_launchd_service() -> Result<()> {
    Command::new("launchctl")
        .args(["bootout", "system", LAUNCHD_PLIST_PATH])
        .status()?;
    fs::remove_file(LAUNCHD_PLIST_PATH)?;
    println!("VPN Helper has been removed from launchd.");
    Ok(())
}

fn disconnect() -> Result<()> {
    Command::new("killall").arg("openconnect").output()?;
    Ok(())
}

fn check_root() -> Result<()> {
    if !nix::unistd::Uid::effective().is_root() {
        return Err(anyhow!("This program must be run as root"));
    }
    Ok(())
}

fn check_required_programs() -> Result<()> {
    for program in ["openconnect", "vpn-slice"] {
        if which::which(program).is_err() {
            return Err(anyhow!("{program} is not installed or not in PATH"));
        }
    }
    Ok(())
}
