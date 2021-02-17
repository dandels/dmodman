use serde::{Deserialize, Serialize};
use super::error::RequestError;
use super::request;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

#[derive(Serialize, Deserialize)]
pub struct Search {
    pub terms: Vec<String>,
    pub exclude_authors: Vec<String>,
    pub exclude_tags: Vec<String>,
    pub include_adult: bool,
    pub took: u64,
    pub total: u64,
    pub results: Vec<SearchResult>,
}

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    pub name: String,
    pub downloads: u64,
    pub endorsements: u64,
    pub url: String,
    pub image: String,
    pub username: String,
    pub user_id: u64,
    pub game_id: u64,
    pub mod_id: u64,
}

#[derive(Serialize, Deserialize)]
pub struct SearchQuery {
    pub terms: Vec<String>,
    pub game_id: u64,
    pub blocked_tags: Vec<u64>,
    pub blocked_authors: Vec<u64>,
    pub include_adult: bool,
}

impl SearchQuery {
    pub async fn send(&self) -> Result<Search, RequestError> {
        request::mod_search(self.format()).await
    }

    fn format(&self) -> String {
        let mut encoded_terms: String = "".to_string();
        for i in 0..self.terms.len() {
            if i > 0 {
                encoded_terms.push(',');
            }
            let encoded = utf8_percent_encode(&self.terms[i], NON_ALPHANUMERIC).to_string();
            encoded_terms.push_str(&encoded);
        }

        let mut tags: String = "".to_string();

        for i in 0..self.blocked_tags.len() {
            if i > 0 {
                tags.push(',');
            }
            tags.push_str(&self.blocked_tags[i].to_string());
        }

        let mut authors: String = "".to_string();

        for i in 0..self.blocked_authors.len() {
            if i > 0 {
                authors.push(',');
            }
            authors.push_str(&self.blocked_authors[i].to_string());
        }

        return format!(
            "?terms={}&game_id={}&blocked_tags={}&blocked_authors={}&include_adult={}",
            encoded_terms, self.game_id, tags, authors, self.include_adult as u8
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::api::search::{SearchQuery};

    #[test]
    fn search_query_format() {
        let sq = SearchQuery {
            terms: vec!["graphic".to_string(), "herbalism".to_string()],
            game_id: 100,
            blocked_tags: vec![1, 2],
            blocked_authors: vec![5, 6],
            include_adult: false,
        };

        let query = "?terms=graphic,herbalism&game_id=100&blocked_tags=1,2&blocked_authors=5,6&include_adult=0";

        assert_eq!(sq.format(), query);
    }
}
