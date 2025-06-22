pub trait StringUtils {
    fn capitalize(&self) -> String;
}

impl StringUtils for str {
    fn capitalize(&self) -> String {
        self.chars()
            .next()
            .map(|c| c.to_uppercase().collect::<String>() + &self[c.len_utf8()..])
            .unwrap_or_default()
    }
}
