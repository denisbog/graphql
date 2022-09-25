use tantivy::{collector::TopDocs, query::QueryParser, Index, LeasedItem, ReloadPolicy, Searcher};

use crate::searchapp::model::Post;

#[tokio::main]
pub async fn search() {
    let dir = tantivy::directory::MmapDirectory::open("index").unwrap();
    let index = Index::open(dir).unwrap();

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
    let results = search_using_index(query_parser, searcher, index);

    let db = sled::open("sled").expect("open");

    let objects = results.iter().map(|id| {
        let post: Post =
            serde_json::from_str(std::str::from_utf8(&db.get(id).unwrap().unwrap()).unwrap())
                .unwrap();
        post
    });
    println!("{}", results.len());
    objects.for_each(|post| println!("{:?}", post));
}

pub fn search_using_index(
    query_parser: QueryParser,
    searcher: LeasedItem<Searcher>,
    index: Index,
) -> Vec<String> {
    let query = query_parser.parse_query("fiat").unwrap();

    let query_results = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();

    query_results
        .iter()
        .map(|(_score, doc_address)| {
            let out = searcher
                .segment_reader(doc_address.segment_ord)
                .fast_fields()
                .bytes(index.schema().get_field("id").unwrap())
                .unwrap();
            return std::str::from_utf8(out.get_bytes(doc_address.doc_id))
                .unwrap()
                .to_string();
        })
        .collect::<Vec<String>>()
}
