use serde::{Deserialize, Serialize};

/// Represents the different strategies for splitting text into chunks.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(tag = "type")]
pub enum Splitter {
    #[default]
    #[serde(rename = "newline")]
    NewLine,
    #[serde(rename = "empty_line")]
    EmptyLine,
    #[serde(rename = "max_words")]
    MaxWords { max_words: usize },
    #[serde(rename = "max_chars")]
    MaxChars { max_chars: usize },
}

impl Splitter {
    /// Splits the given text according to the selected strategy.
    pub fn split(&self, text: &str) -> Vec<String> {
        match self {
            Splitter::NewLine => text
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_string)
                .collect(),
            Splitter::EmptyLine => text
                .split("\n\n")
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .map(str::to_string)
                .collect(),
            Splitter::MaxWords { max_words } => {
                let words = text.split_whitespace().collect::<Vec<_>>();
                words
                    .chunks(*max_words)
                    .map(|chunk| chunk.join(" "))
                    .filter(|chunk| !chunk.is_empty())
                    .collect()
            }
            Splitter::MaxChars { max_chars } => {
                let chars = text.chars().collect::<Vec<_>>();
                chars
                    .chunks(*max_chars)
                    .map(|chunk| chunk.iter().collect::<String>())
                    .filter(|chunk| !chunk.is_empty())
                    .collect()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    // NewLine splitter test cases
    #[rstest]
    #[case::basic_lines("Line 1\nLine 2\nLine 3", vec!["Line 1", "Line 2", "Line 3"])]
    #[case::empty_string("", vec![])]
    #[case::only_whitespace("   \n   \n   ", vec![])]
    #[case::with_tabs_and_spaces("  Line 1  \n\t Line 2 \t\n   Line 3   ", vec!["Line 1", "Line 2", "Line 3"])]
    #[case::newlines_only("\n\n\n", vec![])]
    #[case::mixed_content("Line 1\nLine 2\n\nLine 3\nLine 4\n\n\nLine 5", vec!["Line 1", "Line 2", "Line 3", "Line 4", "Line 5"])]
    fn test_newline_splitter(#[case] input: &str, #[case] expected: Vec<&str>) {
        let splitter = Splitter::NewLine;
        let result = splitter.split(input);
        assert_eq!(result, expected);
    }

    // EmptyLine splitter test cases
    #[rstest]
    #[case::basic_paragraphs("Line 1\nLine 2\n\nLine 3\nLine 4", vec!["Line 1\nLine 2", "Line 3\nLine 4"])]
    #[case::empty_string("", vec![])]
    #[case::single_paragraph("This is a single paragraph\nwith multiple lines\nbut no empty lines", vec!["This is a single paragraph\nwith multiple lines\nbut no empty lines"])]
    #[case::multiple_empty_lines("Para 1\n\n\n\nPara 2\n\n\n\n\nPara 3", vec!["Para 1", "Para 2", "Para 3"])]
    #[case::trailing_content("Line 1\nLine 2\n\nLine 3\nLine 4\n\n\nLine 5", vec!["Line 1\nLine 2", "Line 3\nLine 4", "Line 5"])]
    fn test_empty_line_splitter(#[case] input: &str, #[case] expected: Vec<&str>) {
        let splitter = Splitter::EmptyLine;
        let result = splitter.split(input);
        assert_eq!(result, expected);
    }

    // MaxWords splitter test cases
    #[rstest]
    #[case::basic_chunking("The quick brown fox jumps over the lazy dog", 3, vec!["The quick brown", "fox jumps over", "the lazy dog"])]
    #[case::exact_division("one two three four", 2, vec!["one two", "three four"])]
    #[case::larger_than_text("only three words", 10, vec!["only three words"])]
    #[case::empty_string("", 5, vec![])]
    #[case::extra_whitespace("  word1   word2    word3   word4  ", 2, vec!["word1 word2", "word3 word4"])]
    #[case::single_word_chunks("one two three", 1, vec!["one", "two", "three"])]
    fn test_max_words_splitter(
        #[case] input: &str,
        #[case] max_words: usize,
        #[case] expected: Vec<&str>,
    ) {
        let splitter = Splitter::MaxWords { max_words };
        let result = splitter.split(input);
        assert_eq!(result, expected);
    }

    // MaxChars splitter test cases
    #[rstest]
    #[case::basic_chunking("abcdefghijklmnop", 5, vec!["abcde", "fghij", "klmno", "p"])]
    #[case::exact_division("12345678", 4, vec!["1234", "5678"])]
    #[case::empty_string("", 5, vec![])]
    #[case::larger_than_text("short", 20, vec!["short"])]
    #[case::single_char_chunks("abc", 1, vec!["a", "b", "c"])]
    #[case::unicode_support("Hello 🌍 World 🚀 Test 🦀", 6, vec!["Hello ", "🌍 Worl", "d 🚀 Te", "st 🦀"])]
    fn test_max_chars_splitter(
        #[case] input: &str,
        #[case] max_chars: usize,
        #[case] expected: Vec<&str>,
    ) {
        let splitter = Splitter::MaxChars { max_chars };
        let result = splitter.split(input);
        assert_eq!(result, expected);
    }

    // Edge cases and error conditions
    #[rstest]
    fn test_zero_chunk_size_panics() {
        let max_words_result = std::panic::catch_unwind(|| {
            let splitter = Splitter::MaxWords { max_words: 0 };
            splitter.split("some words here")
        });
        assert!(max_words_result.is_err());

        let max_chars_result = std::panic::catch_unwind(|| {
            let splitter = Splitter::MaxChars { max_chars: 0 };
            splitter.split("hello")
        });
        assert!(max_chars_result.is_err());
    }

    // Serialization test cases
    #[rstest]
    #[case::newline(Splitter::NewLine, r#"{"type":"newline"}"#)]
    #[case::empty_line(Splitter::EmptyLine, r#"{"type":"empty_line"}"#)]
    #[case::max_words(Splitter::MaxWords { max_words: 10 }, r#"{"type":"max_words","max_words":10}"#)]
    #[case::max_chars(Splitter::MaxChars { max_chars: 100 }, r#"{"type":"max_chars","max_chars":100}"#)]
    fn test_serialization(#[case] splitter: Splitter, #[case] expected_json: &str) {
        let json = serde_json::to_string(&splitter).unwrap();
        assert_eq!(json, expected_json);
    }

    // Deserialization test cases
    #[rstest]
    #[case::newline(r#"{"type":"newline"}"#, Splitter::NewLine)]
    #[case::empty_line(r#"{"type":"empty_line"}"#, Splitter::EmptyLine)]
    #[case::max_words(r#"{"type":"max_words","max_words":5}"#, Splitter::MaxWords { max_words: 5 })]
    #[case::max_chars(r#"{"type":"max_chars","max_chars":50}"#, Splitter::MaxChars { max_chars: 50 })]
    fn test_deserialization(#[case] json: &str, #[case] expected: Splitter) {
        let splitter: Splitter = serde_json::from_str(json).unwrap();
        match (&splitter, &expected) {
            (Splitter::NewLine, Splitter::NewLine) => {}
            (Splitter::EmptyLine, Splitter::EmptyLine) => {}
            (Splitter::MaxWords { max_words: a }, Splitter::MaxWords { max_words: b }) => {
                assert_eq!(a, b);
            }
            (Splitter::MaxChars { max_chars: a }, Splitter::MaxChars { max_chars: b }) => {
                assert_eq!(a, b);
            }
            _ => panic!("Deserialized splitter doesn't match expected variant"),
        }
    }

    // Integration test with complex text
    #[rstest]
    fn test_complex_text_integration() {
        let text = "First paragraph line 1\nFirst paragraph line 2\n\nSecond paragraph line 1\nSecond paragraph line 2\n\nThird paragraph";

        // Test all splitter types work with complex text
        let test_cases = vec![
            (Splitter::NewLine, 5),
            (Splitter::EmptyLine, 3),
            (Splitter::MaxWords { max_words: 4 }, 5), // 19 words total, 4 per chunk = 5 chunks
            (Splitter::MaxChars { max_chars: 20 }, 6), // Roughly 6 chunks for this text
        ];

        for (splitter, expected_chunks) in test_cases {
            let result = splitter.split(text);
            assert!(
                !result.is_empty(),
                "Splitter {:?} should produce non-empty result",
                splitter
            );

            match splitter {
                Splitter::NewLine | Splitter::EmptyLine => {
                    assert_eq!(
                        result.len(),
                        expected_chunks,
                        "Splitter {:?} should produce exactly {} chunks",
                        splitter,
                        expected_chunks
                    );
                }
                Splitter::MaxWords { .. } | Splitter::MaxChars { .. } => {
                    assert_eq!(
                        result.len(),
                        expected_chunks,
                        "Splitter {:?} should produce {} chunks, got {}",
                        splitter,
                        expected_chunks,
                        result.len()
                    );
                }
            }
        }
    }

    // Test default implementation
    #[rstest]
    fn test_default() {
        let splitter = Splitter::default();
        assert!(matches!(splitter, Splitter::NewLine));
    }

    // Test Debug trait
    #[rstest]
    fn test_debug_formatting() {
        let splitters = vec![
            Splitter::NewLine,
            Splitter::EmptyLine,
            Splitter::MaxWords { max_words: 5 },
            Splitter::MaxChars { max_chars: 10 },
        ];

        for splitter in splitters {
            let debug_str = format!("{:?}", splitter);
            assert!(!debug_str.is_empty());
        }
    }

    // Property-based testing examples
    #[rstest]
    #[case::small_chunks(1)]
    #[case::medium_chunks(5)]
    #[case::large_chunks(20)]
    fn test_max_words_property_no_empty_chunks(#[case] max_words: usize) {
        let text = "one two three four five six seven eight nine ten";
        let splitter = Splitter::MaxWords { max_words };
        let result = splitter.split(text);

        // Property: No chunk should be empty
        for chunk in &result {
            assert!(!chunk.is_empty());
        }

        // Property: Each chunk should have at most max_words words
        for chunk in &result {
            let word_count = chunk.split_whitespace().count();
            assert!(word_count <= max_words);
        }
    }

    #[rstest]
    #[case::small_chunks(1)]
    #[case::medium_chunks(5)]
    #[case::large_chunks(20)]
    fn test_max_chars_property_no_empty_chunks(#[case] max_chars: usize) {
        let text = "abcdefghijklmnopqrstuvwxyz";
        let splitter = Splitter::MaxChars { max_chars };
        let result = splitter.split(text);

        // Property: No chunk should be empty
        for chunk in &result {
            assert!(!chunk.is_empty());
        }

        // Property: Each chunk should have at most max_chars characters
        for chunk in &result {
            let char_count = chunk.chars().count();
            assert!(char_count <= max_chars);
        }
    }

    // Round-trip testing for serialization
    #[rstest]
    #[case(Splitter::NewLine)]
    #[case(Splitter::EmptyLine)]
    #[case(Splitter::MaxWords { max_words: 42 })]
    #[case(Splitter::MaxChars { max_chars: 123 })]
    fn test_serialization_roundtrip(#[case] original: Splitter) {
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Splitter = serde_json::from_str(&json).unwrap();

        // Test that the functionality is preserved
        let test_text = "Some test text with multiple words and characters.";
        let original_result = original.split(test_text);
        let deserialized_result = deserialized.split(test_text);

        assert_eq!(original_result, deserialized_result);
    }
}
