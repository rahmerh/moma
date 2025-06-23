use anyhow::{Context, bail};
use clap::Args;
use reqwest::Url;

use crate::sources::nexus::Nexus;

#[derive(Args)]
pub struct NxmHandler {
    pub url: String,
}

impl NxmHandler {
    // nxm://skyrimspecialedition/mods/152490/files/638592?key=0Or2IM4l-FXSJjvRogxbMw&expires=1750810470&user_id=191018313
    pub async fn run(&self) -> anyhow::Result<()> {
        let parsed = Nexus::parse_nxm_url(&self.url)?;

        let download_link = Nexus::get_download_link(parsed).await?;

        println!("{}", download_link);

        // get_download_link(&parsed.game, &parsed.mod_id, &parsed.file_id);
        //
        // let client = Client::new();
        // let response = client
        //     .get(url)
        //     .send()
        //     .await
        //     .context("Failed to send request")?;
        //
        // let total_size = response
        //     .content_length()
        //     .context("Failed to get content length")?;
        //
        // let pb = ProgressBar::new(total_size);
        // pb.set_style(
        //     ProgressStyle::default_bar()
        //         .template("{msg:.bold.dim} [{bar:40.cyan/blue}] {bytes:>7}/{total_bytes:7} ({eta})")
        //         .unwrap()
        //         .progress_chars("=>-"),
        // );
        // pb.set_message("Downloading");
        //
        // let output_path = "/tmp/output.zip";
        // let mut dest = BufWriter::new(File::create(output_path)?);
        // let mut stream = response.bytes_stream();
        //
        // while let Some(chunk) = stream.next().await {
        //     let chunk = chunk.context("Error while downloading file")?;
        //     dest.write_all(&chunk).context("Error writing to file")?;
        //     pb.inc(chunk.len() as u64);
        // }
        //
        // pb.finish_with_message("Download complete");
        Ok(())
    }
}
