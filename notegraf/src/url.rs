//! URL utilities.
use crate::{NoteID, Tag};
use std::fmt;
use thiserror::Error;
use url::{ParseError, Url};

/// Error type for Notegraf URL parsing.
#[derive(Error, Debug)]
pub enum NotegrafURLParseError {
    /// Not a valid URL.
    ///
    /// That is, it can't be parse by the `Url` library.
    #[error("Not a valid URL")]
    NotAURL(#[from] ParseError),
    /// Wrong URL scheme, such as HTTP.
    #[error("URL scheme `{0}` not supported")]
    WrongScheme(String),
    /// The URL cannot be a base, such as a base64 encoded image.
    #[error("The URL cannot be a base")]
    CannotBeABase,
    /// Not a valid Notegraf URL.
    ///
    /// For example, the first part of the URL might not have any of the expected value.
    #[error("Syntax error: `{0}`")]
    SyntaxError(String),
}

/// URL type for Notegraf.
#[derive(Debug)]
pub enum NotegrafURL {
    Note(NoteID),
    Tag(Tag),
}

impl NotegrafURL {
    /// Parse a string into a Notegraf URL.
    pub fn parse(link: &str) -> Result<Self, NotegrafURLParseError> {
        let url = match Url::parse(link) {
            Ok(u) => u,
            Err(e) => return Err(NotegrafURLParseError::NotAURL(e)),
        };
        if url.scheme() != "notegraf" {
            return Err(NotegrafURLParseError::WrongScheme(url.scheme().into()));
        }
        let parts = match url.path_segments().map(|c| c.collect::<Vec<_>>()) {
            Some(p) => p,
            None => {
                return Err(NotegrafURLParseError::CannotBeABase);
            }
        };
        if parts.len() != 2 {
            return Err(NotegrafURLParseError::SyntaxError(
                "URL has more than two parts.".into(),
            ));
        }
        match parts[0] {
            "note" => Ok(NotegrafURL::Note(NoteID::new(parts[1].into()))),
            "tag" => Ok(NotegrafURL::Tag(Tag::new(parts[1].into()))),
            _ => Err(NotegrafURLParseError::SyntaxError(
                "First part of the URL not recognized.".into(),
            )),
        }
    }
}

impl fmt::Display for NotegrafURL {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NotegrafURL::Note(id) => write!(f, "notegraf:/note/{}", id),
            NotegrafURL::Tag(tag) => write!(f, "notegraf:/tag/{}", tag),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrong_scheme() {
        let url = NotegrafURL::parse("http://host/note/note1");
        assert!(url.is_err());
        if let NotegrafURLParseError::WrongScheme(s) = url.err().unwrap() {
            assert_eq!(s, "http");
        } else {
            assert!(false);
        }
    }
}
