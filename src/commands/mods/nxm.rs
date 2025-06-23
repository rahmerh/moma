use std::{
    fs::File,
    io::{BufWriter, Write},
};

use anyhow::Context;
use clap::Args;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;

#[derive(Args)]
pub struct NxmHandler {
    pub url: String,
}

impl NxmHandler {
    // nxm://skyrimspecialedition/mods/152490/files/638592?key=0Or2IM4l-FXSJjvRogxbMw&expires=1750810470&user_id=191018313
    pub async fn run(&self) -> anyhow::Result<()> {
        let url = "https://supporter-files.nexus-cdn.com/1704/152490/Viper%20Armor%204k%203BA-152490-1-0-1750096882.zip?md5=K-VQ64tL9aQqe9ny8NvSDQ&expires=1750652508&user_id=191018313";

        let client = Client::new();
        let response = client
            .get(url)
            .send()
            .await
            .context("Failed to send request")?;

        let total_size = response
            .content_length()
            .context("Failed to get content length")?;

        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg:.bold.dim} [{bar:40.cyan/blue}] {bytes:>7}/{total_bytes:7} ({eta})")
                .unwrap()
                .progress_chars("=>-"),
        );
        pb.set_message("Downloading");

        let output_path = "/tmp/output.zip";
        let mut dest = BufWriter::new(File::create(output_path)?);
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Error while downloading file")?;
            dest.write_all(&chunk).context("Error writing to file")?;
            pb.inc(chunk.len() as u64);
        }

        pb.finish_with_message("Download complete");
        Ok(())
    }
}
