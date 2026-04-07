//! 小工具函数

pub(crate) fn normalize_for_embedding(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_trims_and_collapses_whitespace() {
        assert_eq!(normalize_for_embedding("  a \n\t b  "), "a b");
    }
}
