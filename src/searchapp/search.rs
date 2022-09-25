use sled::Db;
use tantivy::{collector::TopDocs, query::QueryParser, Index, LeasedItem, ReloadPolicy, Searcher};

use crate::searchapp::{
    model::Post,
    state::{get_index, get_local_db},
};

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

    pub fn search(&self, search: &str) -> Vec<Post> {
        let results = self.extract_items(search);

        results
            .iter()
            .map(|id| {
                let post: Post = serde_json::from_str(
                    std::str::from_utf8(&self.db.get(id).unwrap().unwrap()).unwrap(),
                )
                .unwrap();
                post
            })
            .collect()
    }

    fn extract_items(&self, search: &str) -> Vec<String> {
        let query = self.query_parser.parse_query(search).unwrap();
        let query_results = self
            .searcher
            .search(&query, &TopDocs::with_limit(10))
            .unwrap();
        query_results
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
            .collect::<Vec<String>>()
    }
}
