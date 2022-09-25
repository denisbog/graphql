use juniper::GraphQLObject;
use sled::Db;
use tantivy::{
    collector::{Count, TopDocs},
    query::QueryParser,
    Index, LeasedItem, ReloadPolicy, Searcher,
};

use crate::searchapp::{
    model::Post,
    state::{get_index, get_local_db},
};

#[derive(GraphQLObject)]
pub struct SearchResults {
    items: Vec<Post>,
    count: i32,
}

pub struct SelectedSearchResults {
    items: Vec<String>,
    count: i32,
}

pub struct SearchEngine {
    index: Index,
    query_parser: QueryParser,
    searcher: LeasedItem<Searcher>,
    db: Db,
}

impl SearchEngine {
    pub fn new() -> SearchEngine {
        let index = get_index();

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()
            .unwrap();

        let searcher = reader.searcher();
        let query_parser = QueryParser::for_index(
            &index,
            vec![index.schema().get_field("description").unwrap()],
        );
        let db = get_local_db();

        SearchEngine {
            index,
            query_parser,
            searcher,
            db,
        }
    }

    pub fn search(&self, search: &str, results: usize, offset: usize) -> SearchResults {
        let results = self.extract_items(search, results, offset);
        let items = results
            .items
            .iter()
            .map(|id| {
                let post: Post = serde_json::from_str(
                    std::str::from_utf8(&self.db.get(id).unwrap().unwrap()).unwrap(),
                )
                .unwrap();
                post
            })
            .collect();
        SearchResults {
            items,
            count: results.count,
        }
    }

    fn extract_items(&self, search: &str, results: usize, offset: usize) -> SelectedSearchResults {
        let query = self.query_parser.parse_query(search).unwrap();

        let count = self.searcher.search(&query, &Count).unwrap();

        let query_results = self
            .searcher
            .search(&query, &TopDocs::with_limit(results).and_offset(offset))
            .unwrap();
        let items = query_results
            .iter()
            .map(|(_score, doc_address)| {
                let out = self
                    .searcher
                    .segment_reader(doc_address.segment_ord)
                    .fast_fields()
                    .bytes(self.index.schema().get_field("id").unwrap())
                    .unwrap();
                return std::str::from_utf8(out.get_bytes(doc_address.doc_id))
                    .unwrap()
                    .to_string();
            })
            .collect::<Vec<String>>();
        SelectedSearchResults {
            items,
            count: i32::try_from(count).unwrap(),
        }
    }
}
