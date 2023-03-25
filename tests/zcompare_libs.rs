use ksp_cfg_formatter::{char_formatter, token_formatter};
use proptest::prelude::*;

proptest! {
    #[ignore = "this takes infinite time"]
    #[test]
    fn compare_libs(text: String) {
        let char_formatter = char_formatter::Formatter::new(char_formatter::Indentation::Tabs, true, char_formatter::LineReturn::LF);
        let char_based = char_formatter.format_text(&text);
        let token_formatter = token_formatter::Formatter::new(token_formatter::Indentation::Tabs, true);
        let token_based = token_formatter.format_text(&text);
        assert_eq!(char_based, token_based, "Outputs were not the same");
    }
}
