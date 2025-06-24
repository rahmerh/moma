pub trait StringUtils {
    fn indent_spaces(&self, amount: usize) -> String;
}

impl<T: AsRef<str>> StringUtils for T {
    fn indent_spaces(&self, amount: usize) -> String {
        let mut result = String::new();
        for _ in 0..amount {
            result.push(' ');
        }
        result.push_str(self.as_ref());
        result
    }
}
