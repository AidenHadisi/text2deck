pub trait Splitter {
    /// Splits the input text into chunks based on the implemented strategy.
    fn split(&self, text: &str) -> Vec<String>;
}

/// A splitter that divides text into chunks based on newline characters.
pub struct NewLineSplitter;

impl Splitter for NewLineSplitter {
    fn split(&self, text: &str) -> Vec<String> {
        text.lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect()
    }
}

/// A splitter that divides text into chunks based on empty lines.
pub struct EmptyLineSplitter;

impl Splitter for EmptyLineSplitter {
    fn split(&self, text: &str) -> Vec<String> {
        text.split("\n\n")
            .map(str::trim)
            .filter(|c| !c.is_empty())
            .map(str::to_string)
            .collect()
    }
}

/// A splitter that divides text into chunks based on maximum word count.
pub struct MaxWordsSplitter {
    max_words: usize,
}

impl MaxWordsSplitter {
    pub fn new(max_words: usize) -> Self {
        Self { max_words }
    }
}

impl Splitter for MaxWordsSplitter {
    fn split(&self, text: &str) -> Vec<String> {
        let words: Vec<_> = text.split_whitespace().collect();
        words
            .chunks(self.max_words)
            .map(|chunk| chunk.join(" "))
            .filter(|chunk| !chunk.is_empty())
            .collect()
    }
}

/// A splitter that divides text into chunks based on maximum character count.
pub struct MaxCharsSplitter {
    max_chars: usize,
}

impl MaxCharsSplitter {
    pub fn new(max_chars: usize) -> Self {
        Self { max_chars }
    }
}

impl Splitter for MaxCharsSplitter {
    fn split(&self, text: &str) -> Vec<String> {
        let chars = text.chars().collect::<Vec<_>>();
        chars
            .chunks(self.max_chars)
            .map(|chunk| chunk.iter().collect::<String>())
            .filter(|chunk| !chunk.is_empty())
            .collect()
    }
}
