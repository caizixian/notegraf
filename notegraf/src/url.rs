//! URL utilities.
use crate::errors::URLParseError;
use crate::NoteID;
use std::fmt;
use url::Url;

/// URL type for Notegraf.
#[derive(Debug)]
pub enum NotegrafURL {
    Note(NoteID),
}

impl NotegrafURL {
    /// Parse a string into a Notegraf URL.
    pub fn parse(link: &str) -> Result<Self, URLParseError> {
        let url = match Url::parse(link) {
            Ok(u) => u,
            Err(e) => return Err(URLParseError::NotAURL(e)),
        };
        if url.scheme() != "notegraf" {
            return Err(URLParseError::WrongScheme(url.scheme().into()));
        }
        let parts = match url.path_segments().map(|c| c.collect::<Vec<_>>()) {
            Some(p) => p,
            None => {
                return Err(URLParseError::CannotBeABase);
            }
        };
        if parts.len() != 2 {
            return Err(URLParseError::SyntaxError(
                "URL has more than two parts.".into(),
            ));
        }
        match parts[0] {
            "note" => Ok(NotegrafURL::Note(NoteID::new(parts[1].into()))),
            _ => Err(URLParseError::SyntaxError(
                "First part of the URL not recognized.".into(),
            )),
        }
    }
}

impl fmt::Display for NotegrafURL {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NotegrafURL::Note(id) => write!(f, "notegraf:/note/{}", id),
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
        if let URLParseError::WrongScheme(s) = url.err().unwrap() {
            assert_eq!(s, "http");
        } else {
            assert!(false);
        }
    }
}
