use crate::splitter::Splitter;

/// Represents a request to create slides from text content.
pub struct Request {
    pub title: String,
    pub content: String,
    pub splitter: Box<dyn Splitter>,
}
