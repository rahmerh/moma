use console::{Color, Style};
use dialoguer::theme::ColorfulTheme;

pub fn default_theme() -> ColorfulTheme {
    let cyan = Color::Color256(81);
    let cyan_style = Style::new().fg(cyan);
    let red_style = Style::new().fg(Color::Red);
    let yellow_style = Style::new().fg(Color::Yellow);
    let white_style = Style::new().fg(Color::White);
    let black_style = Style::new().fg(Color::Black);

    ColorfulTheme {
        prompt_prefix: cyan_style.apply_to("➜".to_string()),
        prompt_suffix: cyan_style.apply_to("›".to_string()),
        prompt_style: white_style.clone(),
        success_prefix: cyan_style.apply_to("✔".to_string()),
        success_suffix: cyan_style.apply_to("·".to_string()),
        error_prefix: red_style.apply_to("✘".to_string()),
        error_style: red_style,
        hint_style: yellow_style,
        values_style: white_style.clone(),
        active_item_prefix: cyan_style.apply_to("❯".to_string()).clone(),
        inactive_item_prefix: black_style.apply_to(" ".to_string()),
        active_item_style: cyan_style,
        inactive_item_style: white_style,
        ..Default::default()
    }
}
