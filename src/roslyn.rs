use std::process::Stdio;

use anyhow::{bail, Context, Result};
use home::home_dir;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{ChildStdout, Command},
};

use crate::{
    download_roslyn::ensure_roslyn_is_installed,
    pipe_stream::{Pipe, PipeStream},
};

#[derive(Serialize, Deserialize)]
struct RoslynResponse {
    #[serde(rename = "pipeName")]
    pipe_name: String,
}

async fn parse_roslyn_response(reader: BufReader<ChildStdout>) -> Result<RoslynResponse> {
    let first_line = reader
        .lines()
        .next_line()
        .await?
        .context("No lines to read")?;

    match serde_json::from_str::<RoslynResponse>(&first_line) {
        Ok(res) => Ok(res),
        Err(_) => bail!("{first_line}"),
    }
}

pub async fn start_roslyn() -> Box<dyn PipeStream> {
    let roslyn_dll = ensure_roslyn_is_installed().expect("Unable to install Roslyn");

    let mut log_dir = home_dir().expect("Unable to find home directory");
    log_dir.push(".roslyn");
    log_dir.push("logs");

    let mut process = Command::new("dotnet")
        .arg(roslyn_dll)
        .arg("--logLevel=Information")
        .arg("--extensionLogDirectory")
        .arg(log_dir)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute command");

    let reader = BufReader::new(process.stdout.take().expect("Failed to capture stdout"));

    let roslyn_response = parse_roslyn_response(reader)
        .await
        .expect("Unable to parse response from server");

    Pipe::connect(&roslyn_response.pipe_name)
        .await
        .expect("Unable to connect to server stream")
}