#[macro_export]
macro_rules! usage_for {
    ($($path:expr),+) => {{
        fn _get_cmd() -> clap::Command {
            <$crate::Cli as clap::CommandFactory>::command()
        }

        let mut cmd = _get_cmd();
        $(
            cmd = cmd
                .find_subcommand($path)
                .unwrap_or_else(|| panic!("Unknown command path: {}", $path))
                .clone();
        )+
        format!(
            "moma {}",
            cmd.render_usage().to_string().trim_start_matches("Usage:").trim()
        )
    }};
}
