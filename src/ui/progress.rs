use crossterm::style::Stylize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub fn display_active_downloads(tracking_dir: &Path) -> anyhow::Result<()> {
    let mp = MultiProgress::new();
    let pb_style = ProgressStyle::default_bar()
        .template("\n{msg} {bar:40.cyan/blue} {bytes}/{total_bytes}")
        .unwrap()
        .progress_chars("=> ");

    let mut active_files: HashSet<u64> = HashSet::new();
    let mut bars: HashMap<u64, (ProgressBar, VecDeque<(Instant, u64)>, PathBuf)> = HashMap::new();

    println!("\n{}", "Active downloads:".cyan().bold().underlined());

    loop {
        for entry in std::fs::read_dir(tracking_dir)? {
            let path = entry?.path();
            if path.extension().map_or(true, |e| e != "json") {
                continue;
            }

            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if let Ok(file_uid) = stem.parse::<u64>() {
                    if !active_files.contains(&file_uid) {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(progress) = serde_json::from_str::<DownloadProgress>(&content)
                            {
                                let pb = mp.add(ProgressBar::new(progress.total_bytes));
                                pb.set_style(pb_style.clone());
                                pb.set_position(progress.progress_bytes);
                                active_files.insert(file_uid);
                                bars.insert(file_uid, (pb, VecDeque::new(), path.clone()));
                            }
                        }
                    }
                }
            }
        }

        for (_uid, (pb, window, path)) in bars.iter_mut() {
            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => {
                    pb.finish_and_clear();
                    continue;
                }
            };

            let progress: DownloadProgress = match serde_json::from_str(&content) {
                Ok(p) => p,
                Err(_) => continue,
            };

            pb.set_length(progress.total_bytes);
            pb.set_position(progress.progress_bytes);

            let now = Instant::now();
            let bytes = progress.progress_bytes;
            window.push_back((now, bytes));

            if window.len() > 20 {
                window.pop_front();
            }

            let speed_bps = if let (Some((old_t, old_b)), Some((new_t, new_b))) =
                (window.front(), window.back())
            {
                let byte_delta = new_b.saturating_sub(*old_b);
                let time_delta = (*new_t - *old_t).as_secs_f64();
                if time_delta > 0.0 {
                    byte_delta as f64 / time_delta
                } else {
                    0.0
                }
            } else {
                0.0
            };

            pb.set_message(format!(
                "{}\n{:.2} MB/s",
                progress.file_name.bold().underlined(),
                speed_bps / 1_048_576.0
            ));
        }

        std::thread::sleep(Duration::from_millis(500));
    }
}
