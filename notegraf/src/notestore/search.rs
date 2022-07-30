pub struct SearchRequest {
    pub(super) lexemes: Vec<String>,
    pub(super) lexemes_excluded: Vec<String>,
    pub(super) tags: Vec<String>,
    pub(super) tags_excluded: Vec<String>,
    pub(super) orphan: bool,
    pub(super) no_tag: bool,
    pub(super) limit: Option<u64>,
}

impl SearchRequest {
    pub(super) fn sort_by_created_at(&self) -> bool {
        self.lexemes.is_empty()
    }
}

static DEFAULT_LIMIT: u64 = 10;

fn parse_query(query: &str) -> SearchRequest {
    let parts: Vec<&str> = query.split(' ').collect();
    let mut lexemes = vec![];
    let mut lexemes_excluded = vec![];
    let mut tags = vec![];
    let mut tags_excluded = vec![];
    let mut orphan = false;
    let mut limit = None;
    let mut no_tag = false;
    let mut no_limit = false;
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
            } else if stripped == "nolimit" {
                no_limit = true;
            } else if let Some(limit_str) = stripped.strip_prefix("limit=") {
                limit = limit_str.parse::<u64>().ok();
            }
        } else if let Some(negation) = part.strip_prefix('-') {
            if let Some(stripped) = negation.strip_prefix('#') {
                if !stripped.is_empty() {
                    tags_excluded.push(stripped.to_owned());
                }
            } else if !negation.is_empty() {
                lexemes_excluded.push(negation.to_owned());
            }
        } else if !part.is_empty() {
            lexemes.push(part.to_owned());
        }
    }
    if lexemes.is_empty() && limit.is_none() {
        limit = Some(DEFAULT_LIMIT);
    }
    if no_limit {
        limit = None;
    }
    SearchRequest {
        lexemes,
        lexemes_excluded,
        tags,
        tags_excluded,
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

    #[test]
    fn empty_limit() {
        let sr: SearchRequest = "".into();
        assert_eq!(sr.limit, Some(DEFAULT_LIMIT));
    }

    #[test]
    fn empty_limit_override() {
        let sr: SearchRequest = "!limit=32".into();
        assert_eq!(sr.limit, Some(32));
    }

    #[test]
    fn tag_only_limit() {
        let sr: SearchRequest = "#tag".into();
        assert_eq!(sr.limit, Some(DEFAULT_LIMIT));
    }

    #[test]
    fn tag_only_limit_override() {
        let sr: SearchRequest = "!limit=32 #tag".into();
        assert_eq!(sr.limit, Some(32));
    }

    #[test]
    fn lexeme_only_limit() {
        let sr: SearchRequest = "foo".into();
        assert_eq!(sr.limit, None);
    }

    #[test]
    fn lexeme_only_limit_override() {
        let sr: SearchRequest = "foo !limit=5".into();
        assert_eq!(sr.limit, Some(5));
    }

    #[test]
    fn exclude_lexemes() {
        let sr: SearchRequest = "-foo bar".into();
        assert_eq!(sr.lexemes, vec!["bar".to_owned()]);
        assert_eq!(sr.lexemes_excluded, vec!["foo".to_owned()]);
    }

    #[test]
    fn exclude_tags() {
        let sr: SearchRequest = "-#foo #bar".into();
        assert_eq!(sr.tags, vec!["bar".to_owned()]);
        assert_eq!(sr.tags_excluded, vec!["foo".to_owned()]);
    }

    #[test]
    fn nolimit() {
        let sr: SearchRequest = "!nolimit".into();
        assert_eq!(sr.limit, None);
    }

    #[test]
    fn nolimit_higher_precedence() {
        let sr: SearchRequest = "!limit=512 !nolimit".into();
        assert_eq!(sr.limit, None);

        let sr: SearchRequest = "!nolimit !limit=512".into();
        assert_eq!(sr.limit, None);
    }
}
