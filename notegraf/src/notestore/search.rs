pub struct SearchRequest {
    pub(super) lexemes: Vec<String>,
    pub(super) tags: Vec<String>,
    pub(super) orphan: bool,
    pub(super) no_tag: bool,
    pub(super) limit: Option<u64>,
}

impl SearchRequest {
    pub(super) fn sort_by_created_at(&self) -> bool {
        self.lexemes.is_empty()
    }
}

fn parse_query(query: &str) -> SearchRequest {
    let parts: Vec<&str> = query.split(' ').collect();
    let mut lexemes = vec![];
    let mut tags = vec![];
    let mut orphan = false;
    let mut limit = None;
    let mut no_tag = false;
    for part in parts {
        if let Some(stripped) = part.strip_prefix('#') {
            if !stripped.is_empty() {
                tags.push(stripped.to_owned());
            }
        } else if let Some(stripped) = part.strip_prefix('!') {
            if stripped == "orphan" {
                orphan = true;
            } else if stripped == "notag" {
                no_tag = true;
            }
        } else if !part.is_empty() {
            lexemes.push(part.to_owned());
        }
    }
    if lexemes.is_empty() {
        limit = Some(10);
    }
    SearchRequest {
        lexemes,
        tags,
        orphan,
        no_tag,
        limit,
    }
}

impl From<String> for SearchRequest {
    fn from(query: String) -> SearchRequest {
        parse_query(&query)
    }
}

impl From<&str> for SearchRequest {
    fn from(query: &str) -> SearchRequest {
        parse_query(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_recent() {
        let sr: SearchRequest = "".into();
        assert!(sr.sort_by_created_at());
    }

    #[test]
    fn whitespace_is_recent() {
        let sr: SearchRequest = "  ".into();
        assert!(sr.sort_by_created_at());
    }

    #[test]
    fn one_tag() {
        let sr: SearchRequest = "#foo".into();
        assert!(!sr.tags.is_empty());
        assert_eq!(sr.tags, vec!["foo".to_owned()]);
    }

    #[test]
    fn two_tags() {
        let sr: SearchRequest = "#foo  #bar ".into();
        assert!(!sr.tags.is_empty());
        assert_eq!(sr.tags, vec!["foo".to_owned(), "bar".to_owned()]);
    }

    #[test]
    fn one_lexeme() {
        let sr: SearchRequest = "fizz ".into();
        assert!(!sr.sort_by_created_at());
        assert!(sr.tags.is_empty());
        assert!(!sr.lexemes.is_empty());
        assert_eq!(sr.lexemes, vec!["fizz".to_owned()]);
    }

    #[test]
    fn lexemes() {
        let sr: SearchRequest = "fizz buzz ".into();
        assert!(!sr.sort_by_created_at());
        assert!(sr.tags.is_empty());
        assert!(!sr.lexemes.is_empty());
        assert_eq!(sr.lexemes, vec!["fizz".to_owned(), "buzz".to_owned()]);
    }

    #[test]
    fn orphan_recent() {
        let sr: SearchRequest = "!orphan".into();
        assert!(sr.sort_by_created_at());
        assert!(sr.orphan);
    }

    #[test]
    fn orphan_lexemes() {
        let sr: SearchRequest = "!orphan foo".into();
        assert!(!sr.sort_by_created_at());
        assert_eq!(sr.lexemes, vec!["foo".to_owned()]);
        assert!(sr.orphan);
    }

    #[test]
    fn orphan_mixed() {
        let sr: SearchRequest = "!orphan foo #bar".into();
        assert!(!sr.sort_by_created_at());
        assert_eq!(sr.lexemes, vec!["foo".to_owned()]);
        assert_eq!(sr.tags, vec!["bar".to_owned()]);
        assert!(sr.orphan);
    }
}
