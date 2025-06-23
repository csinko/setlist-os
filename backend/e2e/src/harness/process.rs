//! spawn_with_logs – runs a binary, colour-prefixes every stdout/stderr line,
//! kills the process automatically when the `Child` is dropped.

use anyhow::{Context, Result};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    task::JoinHandle,
};

pub fn spawn_with_logs(
    tag: &str,
    bin: &str,
    envs: &[(&str, &str)],
    color: u8,
) -> Result<(Child, JoinHandle<()>)> {
    let mut cmd = Command::new(bin);
    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);

    for &(k, v) in envs { cmd.env(k, v); }

    let mut child = cmd.spawn().with_context(|| format!("spawn {tag}"))?;

    println!(
        "\x1b[1;{color}m{tag} launched (pid={})\x1b[0m",
        child.id().unwrap_or(0)
    );

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let prefix = format!("\x1b[1;{color}m{tag}\x1b[0m");

    let handle = tokio::spawn(async move {
        let mut out = BufReader::new(stdout).lines();
        let mut err = BufReader::new(stderr).lines();
        loop {
            tokio::select! {
                l = out.next_line() => match l {
                    Ok(Some(line)) => println!("{prefix} {line}"),
                    _               => break,
                },
                l = err.next_line() => match l {
                    Ok(Some(line)) => println!("{prefix} {line}"),
                    _               => break,
                },
            }
        }
    });

    Ok((child, handle))
}

