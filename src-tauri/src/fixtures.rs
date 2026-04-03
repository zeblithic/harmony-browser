use harmony_content::book::{BookStore, MemoryBookStore};
use harmony_content::bundle::BundleBuilder;
use harmony_content::cid::ContentId;

pub struct Fixture {
    pub cid: ContentId,
    pub data: Vec<u8>,
}

const ALL_NAMED_PATHS: &[&str] = &[
    "harmony/content/wiki/hello",
    "harmony/content/wiki/trust-demo",
    "harmony/content/plain/example",
];

/// Resolve a CID to a fixture by checking all known fixtures.
pub fn resolve_by_cid(target: &ContentId) -> Option<Fixture> {
    for path in ALL_NAMED_PATHS {
        if let Some(fixture) = resolve_named(path) {
            if fixture.cid == *target {
                return Some(fixture);
            }
        }
    }
    None
}

/// Resolve a named path to a fixture.
/// Returns None if the path is unknown.
pub fn resolve_named(key_expr: &str) -> Option<Fixture> {
    match key_expr {
        "harmony/content/wiki/hello" => Some(build_markdown_fixture(
            b"# Hello, Harmony!\n\nWelcome to the decentralized web.\n\n## What is this?\n\nThis is a content-addressed document. Its identity is its hash.\nNo servers, no cookies, no tracking.\n",
        )),
        "harmony/content/wiki/trust-demo" => Some(build_markdown_fixture(
            b"# Trust Demo\n\nThis content comes from a **trusted author**.\n\nImages and media would load automatically at this trust level.\n",
        )),
        "harmony/content/plain/example" => Some(build_plain_fixture(
            b"Just plain bytes.\nNo formatting, no markup.\nContent-addressed and tamper-proof.",
        )),
        _ => None,
    }
}

// BundleBuilder inlines blob data at build time, so the MemoryBookStore
// is only needed to compute the CID. Safe to drop after build().
fn build_markdown_fixture(content: &[u8]) -> Fixture {
    let mut store = MemoryBookStore::new();
    let blob_cid = store.insert(content).unwrap();
    let mut builder = BundleBuilder::new();
    builder.add(blob_cid);
    // with_metadata(total_size, chunk_count, timestamp, mime_8bytes)
    builder.with_metadata(content.len() as u64, 1, 1000, *b"text/md\0");
    let (data, cid) = builder.build().unwrap();
    Fixture { cid, data }
}

fn build_plain_fixture(content: &[u8]) -> Fixture {
    let mut store = MemoryBookStore::new();
    let blob_cid = store.insert(content).unwrap();
    let mut builder = BundleBuilder::new();
    builder.add(blob_cid);
    // with_metadata(total_size, chunk_count, timestamp, mime_8bytes)
    builder.with_metadata(content.len() as u64, 1, 1000, *b"text/pln");
    let (data, cid) = builder.build().unwrap();
    Fixture { cid, data }
}
