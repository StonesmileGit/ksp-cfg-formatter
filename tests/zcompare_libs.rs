use ksp_cfg_formatter::{char_formatter, token_formatter, Indentation, LineReturn};
use proptest::prelude::*;

proptest! {
    #[ignore = "this takes infinite time"]
    #[test]
    fn compare_libs(text: String) {
        let char_formatter = char_formatter::Formatter::new(Indentation::Tabs, true, LineReturn::LF);
        let char_based = char_formatter.format_text(&text);
        let token_formatter = token_formatter::Formatter::new(Indentation::Tabs, true, LineReturn::LF);
        let token_based = token_formatter.format_text(&text);
        assert_eq!(char_based, token_based, "Outputs were not the same");
    }
}
