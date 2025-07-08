#[macro_export]
macro_rules! usage_for {
    ($($path:expr),+) => {{
        fn _get_cmd() -> clap::Command {
            <$crate::cli::Cli as clap::CommandFactory>::command()
        }

        let mut cmd = _get_cmd();
        $(
            cmd = cmd.find_subcommand($path)
                .expect(&format!("Unknown command path: {}", $path))
                .clone();
        )+
        format!(
            "moma {}",
            cmd.render_usage().to_string().trim_start_matches("Usage:").trim()
        )
    }};
}

#[cfg(test)]
mod tests {
    use crate::cli::Cli;

    #[test]
    fn usage_for_macro_should_return_correct_command_usage() {
        // Act
        let usage = usage_for!(Cli::INIT);

        // Assert
        assert_eq!(usage, "moma init");
    }

    #[test]
    fn usage_for_macro_should_include_value_option_if_present() {
        // Act
        let usage = usage_for!(Cli::CONNECT);

        // Assert
        assert_eq!(usage, "moma connect <SOURCE>");
    }

    #[test]
    #[should_panic]
    fn usage_for_should_panic_if_unknown_command_given() {
        let _ = usage_for!("invalid");
    }
}
