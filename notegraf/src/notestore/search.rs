pub struct SearchRequest {
    pub(super) full_text: String,
}

impl From<String> for SearchRequest {
    fn from(query: String) -> SearchRequest {
        SearchRequest { full_text: query }
    }
}

impl From<&str> for SearchRequest {
    fn from(query: &str) -> SearchRequest {
        SearchRequest {
            full_text: query.to_owned(),
        }
    }
}
