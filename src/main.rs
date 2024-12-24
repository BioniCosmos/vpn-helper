mod config;

use anyhow::{anyhow, Result};
use clap::{arg, Command as ClapCommand};
use config::Config;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::{env, process::Stdio};
use totp_rs::{Algorithm, Secret, TOTP};

const MODE_WATCH: &str = "watch";
const MODE_ONCE: &str = "once";

fn main() -> Result<()> {
    let matches = ClapCommand::new("vpn-helper")
        .subcommand(
            ClapCommand::new("connect")
                .about("Connect to VPN")
                .arg(arg!(-m --mode [MODE] "The running mode")),
        )
        .subcommand(ClapCommand::new("register").about("Register as macOS launchctl service"))
        .subcommand(ClapCommand::new("disconnect").about("Disconnect the VPN"))
        .get_matches();
    match matches.subcommand() {
        Some(("connect", sub_matches)) => connect_vpn(
            sub_matches
                .get_one::<String>("mode")
                .map(|s| s.as_str())
                .unwrap_or(MODE_ONCE),
        ),
        Some(("register", _)) => register_launchctl(),
        Some(("disconnect", _)) => disconnect(),
        _ => Err(anyhow!(
            "Invalid command. Use 'connect' or 'register' or 'disconnect'."
        )),
    }
}

fn connect_vpn(mode: &str) -> Result<()> {
    check_root()?;
    check_required_programs()?;
    dotenv::dotenv()?;
    let config = Config::from_env()?;
    let secret = Secret::Encoded(config.totp_secret);
    let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret.to_bytes()?)?;
    let token = totp.generate_current()?;
    let mut child = Command::new("openconnect")
        .arg("--user")
        .arg(&config.username)
        .arg("--passwd-on-stdin")
        .arg("--background")
        .arg("--script")
        .arg(format!("vpn-slice {}", config.route_cidr))
        .arg(&config.host)
        .stdin(Stdio::piped())
        .spawn()?;
    let mut stdin = child.stdin.take().ok_or(anyhow!("Failed to open stdin"))?;
    write!(&mut stdin, "{}{}", config.password, token)?;
    println!("OpenConnect is running on {}.", child.id());
    if mode == MODE_WATCH {
        child.wait()?;
    }
    Ok(())
}

fn register_launchctl() -> Result<()> {
    println!("Registering as macOS launchctl service...");

    let current_exe = env::current_exe()?;
    let current_exe_str = current_exe
        .to_str()
        .ok_or_else(|| anyhow!("Invalid path"))?;
    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>KeepAlive</key>
	<true/>
	<key>Label</key>
	<string>VPN Helper</string>
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
	</array>
	<key>RunAtLoad</key>
	<true/>
</dict>
</plist>"#,
        current_exe_str
    );

    let plist_path = "/Library/LaunchDaemons/com.moecm.vpn-helper.plist";
    let mut file = File::create(plist_path)?;
    file.write_all(plist_content.as_bytes())?;

    Command::new("sudo")
        .args(&["chown", "root:wheel", plist_path])
        .status()?;
    Command::new("sudo")
        .args(&["chmod", "644", plist_path])
        .status()?;

    Command::new("sudo")
        .args(&["launchctl", "load", "-w", plist_path])
        .status()?;

    println!("VPN Helper has been registered as a launchd service.");
    println!("The service will start automatically on system boot.");
    println!("To start the service immediately, run:");
    println!("sudo launchctl start com.user.vpn-helper");

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
    for program in &["openconnect", "vpn-slice"] {
        if which::which(program).is_err() {
            return Err(anyhow!("{program} is not installed or not in PATH"));
        }
    }
    Ok(())
}
