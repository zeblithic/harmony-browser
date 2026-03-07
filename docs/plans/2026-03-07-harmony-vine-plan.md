# Harmony Vine Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a decentralized micro-video feed (Vine) as the first real application in harmony-browser, proving out the content-addressed stack with fixture data.

**Architecture:** VineDescriptor content type in harmony-content. Vine key expressions in harmony-zenoh. VineFeed sans-I/O state machine in harmony-browser crate. VinePlayer/VineCard/VineFeed Svelte components in the Tauri app. Fixture vines for demo until real networking lands.

**Tech Stack:** Rust (harmony crates), Tauri v2, Svelte 5 (runes), TypeScript, vitest, postcard (binary serialization)

**Repos:**
- Tasks 1-4: `/Users/zeblith/work/zeblithic/harmony` (Rust crates)
- Tasks 5-8: `/Users/zeblith/work/zeblithic/harmony-browser` (Tauri app + Svelte)

**Design doc:** `docs/plans/2026-03-07-harmony-vine-design.md`

---

## Phase 1: Rust Foundation (harmony repo)

### Task 1: VineDescriptor and MIME Constants in harmony-content

**Files:**
- Create: `crates/harmony-content/src/vine.rs`
- Modify: `crates/harmony-content/src/lib.rs` (add `pub mod vine;`)
- Modify: `crates/harmony-content/Cargo.toml` (add `postcard` + `serde` deps if not already present)

**Context:** VineDescriptor is the metadata for a published vine -- who created it, when, optional title, and whether it's a reshare. Serialized into a descriptor blob inside a vine bundle. MIME constants identify vine content in InlineMetadata's 8-byte field.

**Step 1: Write the failing test**

```rust
// In crates/harmony-content/src/vine.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vine_descriptor_round_trip() {
        let desc = VineDescriptor {
            creator_address: [0xAB; 16],
            created_at: 1_700_000_000,
            title: Some("funny cat".into()),
            reshare_of: None,
        };
        let bytes = desc.to_bytes();
        let parsed = VineDescriptor::from_bytes(&bytes).unwrap();
        assert_eq!(parsed, desc);
    }

    #[test]
    fn vine_descriptor_reshare_round_trip() {
        let desc = VineDescriptor {
            creator_address: [0xCD; 16],
            created_at: 1_700_000_001,
            title: None,
            reshare_of: Some([0xFF; 32]),
        };
        let bytes = desc.to_bytes();
        let parsed = VineDescriptor::from_bytes(&bytes).unwrap();
        assert_eq!(parsed, desc);
    }

    #[test]
    fn vine_descriptor_title_too_long() {
        let desc = VineDescriptor {
            creator_address: [0; 16],
            created_at: 0,
            title: Some("x".repeat(141)),
            reshare_of: None,
        };
        assert!(desc.validate().is_err());
    }

    #[test]
    fn mime_constants_are_8_bytes() {
        assert_eq!(MIME_VINE_VIDEO.len(), 8);
        assert_eq!(MIME_VINE_COMPILATION.len(), 8);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd /Users/zeblith/work/zeblithic/harmony && cargo test -p harmony-content vine`
Expected: FAIL -- module `vine` doesn't exist

**Step 3: Write minimal implementation**

```rust
// crates/harmony-content/src/vine.rs
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Serialize, Deserialize};

/// 8-byte MIME tag for vine video content.
pub const MIME_VINE_VIDEO: [u8; 8] = *b"vine/vid";

/// 8-byte MIME tag for vine compilation bundles.
pub const MIME_VINE_COMPILATION: [u8; 8] = *b"vine/cmp";

/// Maximum title length in characters.
pub const MAX_TITLE_LEN: usize = 140;

/// Metadata for a published vine. Serialized into a descriptor blob
/// that becomes part of the vine bundle alongside the video blob.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VineDescriptor {
    pub creator_address: [u8; 16],
    pub created_at: u64,
    pub title: Option<String>,
    pub reshare_of: Option<[u8; 32]>,
}

impl VineDescriptor {
    pub fn to_bytes(&self) -> Vec<u8> {
        postcard::to_allocvec(self).expect("VineDescriptor serialization cannot fail")
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(data)
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if let Some(ref title) = self.title {
            if title.len() > MAX_TITLE_LEN {
                return Err("title exceeds 140 characters");
            }
        }
        Ok(())
    }
}
```

Add to `crates/harmony-content/src/lib.rs`:
```rust
pub mod vine;
```

Add to `crates/harmony-content/Cargo.toml` `[dependencies]` (if not already present):
```toml
postcard = { version = "1", default-features = false, features = ["alloc"] }
serde = { version = "1", default-features = false, features = ["derive", "alloc"] }
```

**Step 4: Run test to verify it passes**

Run: `cd /Users/zeblith/work/zeblithic/harmony && cargo test -p harmony-content vine`
Expected: PASS (4 tests)

**Step 5: Run full workspace tests**

Run: `cd /Users/zeblith/work/zeblithic/harmony && cargo test --workspace`
Expected: PASS -- no regressions

**Step 6: Commit**

```bash
cd /Users/zeblith/work/zeblithic/harmony
git add crates/harmony-content/src/vine.rs crates/harmony-content/src/lib.rs crates/harmony-content/Cargo.toml
git commit -m "feat(content): add VineDescriptor and vine MIME constants"
```

---

### Task 2: Vine Key Expressions in harmony-zenoh

**Files:**
- Modify: `crates/harmony-zenoh/src/keyspace.rs`

**Context:** Vine announcements and reactions use Zenoh key expressions. Following a creator = subscribing to their announcement key expression. Builders follow the existing pattern in keyspace.rs (`channel_msg_key()`, `presence_key()`). All builders validate fields (reject `/` chars).

Key expression patterns:
- Announce: `harmony/vines/{address_hex}/announce/{bundle_cid_hex}`
- Announce subscription: `harmony/vines/{address_hex}/announce/**`
- Reaction: `harmony/vines/{target_creator_hex}/reactions/{reactor_hex}`
- Compilation: `harmony/vines/{address_hex}/compilations/{cid_hex}`

**Step 1: Write the failing test**

Add to the existing `#[cfg(test)]` block in keyspace.rs:

```rust
#[test]
fn vine_announce_key_valid() {
    let addr = "ab".repeat(16);
    let cid = "cd".repeat(32);
    let key = vine_announce_key(&addr, &cid).unwrap();
    assert_eq!(key, format!("harmony/vines/{}/announce/{}", addr, cid));
}

#[test]
fn vine_announce_sub_valid() {
    let addr = "ab".repeat(16);
    let sub = vine_announce_sub(&addr).unwrap();
    assert_eq!(sub, format!("harmony/vines/{}/announce/**", addr));
}

#[test]
fn vine_announce_key_rejects_slash() {
    assert!(vine_announce_key("bad/addr", &"cd".repeat(32)).is_err());
}

#[test]
fn vine_reaction_key_valid() {
    let creator = "aa".repeat(16);
    let reactor = "bb".repeat(16);
    let key = vine_reaction_key(&creator, &reactor).unwrap();
    assert_eq!(key, format!("harmony/vines/{}/reactions/{}", creator, reactor));
}

#[test]
fn vine_compilation_key_valid() {
    let addr = "aa".repeat(16);
    let cid = "cc".repeat(32);
    let key = vine_compilation_key(&addr, &cid).unwrap();
    assert_eq!(key, format!("harmony/vines/{}/compilations/{}", addr, cid));
}
```

**Step 2: Run test to verify it fails**

Run: `cd /Users/zeblith/work/zeblithic/harmony && cargo test -p harmony-zenoh vine`
Expected: FAIL -- functions don't exist

**Step 3: Write minimal implementation**

Add to `crates/harmony-zenoh/src/keyspace.rs` (use whatever field validation helper exists -- likely `reject_slashes` or similar):

```rust
/// Build a vine announcement key expression.
pub fn vine_announce_key(creator_addr_hex: &str, bundle_cid_hex: &str) -> Result<String, ZenohError> {
    reject_slashes(creator_addr_hex, "creator_addr_hex")?;
    reject_slashes(bundle_cid_hex, "bundle_cid_hex")?;
    Ok(format!("harmony/vines/{}/announce/{}", creator_addr_hex, bundle_cid_hex))
}

/// Build a vine announcement subscription pattern.
pub fn vine_announce_sub(creator_addr_hex: &str) -> Result<String, ZenohError> {
    reject_slashes(creator_addr_hex, "creator_addr_hex")?;
    Ok(format!("harmony/vines/{}/announce/**", creator_addr_hex))
}

/// Build a vine reaction key expression.
pub fn vine_reaction_key(target_creator_hex: &str, reactor_hex: &str) -> Result<String, ZenohError> {
    reject_slashes(target_creator_hex, "target_creator_hex")?;
    reject_slashes(reactor_hex, "reactor_hex")?;
    Ok(format!("harmony/vines/{}/reactions/{}", target_creator_hex, reactor_hex))
}

/// Build a vine compilation key expression.
pub fn vine_compilation_key(creator_addr_hex: &str, compilation_cid_hex: &str) -> Result<String, ZenohError> {
    reject_slashes(creator_addr_hex, "creator_addr_hex")?;
    reject_slashes(compilation_cid_hex, "compilation_cid_hex")?;
    Ok(format!("harmony/vines/{}/compilations/{}", creator_addr_hex, compilation_cid_hex))
}
```

Note: Check the exact name of the field validation helper. It may be called `validate_field` or similar. Match the existing pattern.

**Step 4: Run test to verify it passes**

Run: `cd /Users/zeblith/work/zeblithic/harmony && cargo test -p harmony-zenoh vine`
Expected: PASS (5 tests)

**Step 5: Commit**

```bash
cd /Users/zeblith/work/zeblithic/harmony
git add crates/harmony-zenoh/src/keyspace.rs
git commit -m "feat(zenoh): add vine announcement and reaction key expressions"
```

---

### Task 3: MimeHint::Video in harmony-browser Crate

**Files:**
- Modify: `crates/harmony-browser/src/types.rs`

**Context:** `MimeHint` detects content types from InlineMetadata's 8-byte MIME field. Adding `Video` lets the browser recognize vine video content and route it to the video player.

**Step 1: Write the failing test**

```rust
#[test]
fn mime_hint_video() {
    let hint = MimeHint::from_mime_bytes(*b"vine/vid");
    assert!(matches!(hint, MimeHint::Video));
}
```

Note: Check how `MimeHint` is currently parsed -- it may be `from_mime_bytes`, `from_raw`, or a `From` impl. Match the existing pattern.

**Step 2: Run test to verify it fails**

Run: `cd /Users/zeblith/work/zeblithic/harmony && cargo test -p harmony-browser mime_hint_video`
Expected: FAIL -- `MimeHint::Video` doesn't exist

**Step 3: Write minimal implementation**

In `crates/harmony-browser/src/types.rs`:

Add variant to `MimeHint`:
```rust
pub enum MimeHint {
    Markdown,
    PlainText,
    Image(ImageFormat),
    Video,              // <-- NEW
    HarmonyApp,
    Unknown([u8; 8]),
}
```

Add match arm in the MIME parsing function:
```rust
b"vine/vid" => MimeHint::Video,
```

**Step 4: Run test to verify it passes**

Run: `cd /Users/zeblith/work/zeblithic/harmony && cargo test -p harmony-browser`
Expected: PASS -- all existing + new test

**Step 5: Commit**

```bash
cd /Users/zeblith/work/zeblithic/harmony
git add crates/harmony-browser/src/types.rs
git commit -m "feat(browser): add MimeHint::Video variant for vine content"
```

---

### Task 4: VineFeed State Machine in harmony-browser Crate

**Files:**
- Create: `crates/harmony-browser/src/vine.rs`
- Modify: `crates/harmony-browser/src/lib.rs` (add `pub mod vine; pub use vine::*;`)
- Modify: `crates/harmony-browser/Cargo.toml` (add `hex` dep if not present)

**Context:** VineFeed is a sans-I/O state machine, separate from BrowserCore. It tracks followed creators, feed items, and viewed state. The Tauri shell composes VineFeed with BrowserCore. VineFeed emits actions like Subscribe/Unsubscribe (which the shell forwards to Zenoh) and FeedUpdated (which the shell sends to the frontend).

**Step 1: Write the failing tests**

```rust
// crates/harmony-browser/src/vine.rs
#[cfg(test)]
mod tests {
    use super::*;

    fn addr(byte: u8) -> [u8; 16] { [byte; 16] }

    fn make_item(creator: [u8; 16], timestamp: u64, seed: u8) -> VineFeedItem {
        VineFeedItem {
            bundle_cid: [seed; 32],
            video_cid: [seed + 100; 32],
            creator,
            timestamp,
            title: None,
            reshare_of: None,
        }
    }

    #[test]
    fn follow_emits_subscribe() {
        let mut feed = VineFeed::new();
        let actions = feed.handle_event(VineEvent::FollowCreator { address: addr(0xAA) });
        assert_eq!(actions.len(), 1);
        match &actions[0] {
            VineAction::Subscribe { key_expr } => {
                assert!(key_expr.contains(&"aa".repeat(16)));
                assert!(key_expr.ends_with("/**"));
            }
            other => panic!("Expected Subscribe, got {:?}", other),
        }
    }

    #[test]
    fn unfollow_emits_unsubscribe() {
        let mut feed = VineFeed::new();
        feed.handle_event(VineEvent::FollowCreator { address: addr(0xAA) });
        let actions = feed.handle_event(VineEvent::UnfollowCreator { address: addr(0xAA) });
        assert!(actions.iter().any(|a| matches!(a, VineAction::Unsubscribe { .. })));
    }

    #[test]
    fn unfollow_unknown_is_noop() {
        let mut feed = VineFeed::new();
        let actions = feed.handle_event(VineEvent::UnfollowCreator { address: addr(0xBB) });
        assert!(actions.is_empty());
    }

    #[test]
    fn vine_announced_adds_to_feed() {
        let mut feed = VineFeed::new();
        feed.handle_event(VineEvent::FollowCreator { address: addr(0xAA) });
        let item = make_item(addr(0xAA), 1000, 1);
        let actions = feed.handle_event(VineEvent::VineAnnounced { item });
        assert!(actions.iter().any(|a| matches!(a, VineAction::FeedUpdated)));
    }

    #[test]
    fn vine_from_unfollowed_creator_ignored() {
        let mut feed = VineFeed::new();
        let item = make_item(addr(0xCC), 1000, 1);
        let actions = feed.handle_event(VineEvent::VineAnnounced { item });
        assert!(actions.is_empty());
    }

    #[test]
    fn mark_viewed() {
        let mut feed = VineFeed::new();
        feed.handle_event(VineEvent::FollowCreator { address: addr(0xAA) });
        feed.handle_event(VineEvent::VineAnnounced { item: make_item(addr(0xAA), 1000, 1) });

        assert_eq!(feed.new_items().len(), 1);
        feed.handle_event(VineEvent::MarkViewed { bundle_cid: [1; 32] });
        assert_eq!(feed.new_items().len(), 0);
        assert_eq!(feed.archive_items().len(), 1);
    }

    #[test]
    fn mark_all_viewed() {
        let mut feed = VineFeed::new();
        feed.handle_event(VineEvent::FollowCreator { address: addr(0xAA) });
        feed.handle_event(VineEvent::VineAnnounced { item: make_item(addr(0xAA), 100, 1) });
        feed.handle_event(VineEvent::VineAnnounced { item: make_item(addr(0xAA), 200, 2) });

        assert_eq!(feed.new_items().len(), 2);
        feed.handle_event(VineEvent::MarkAllViewed);
        assert_eq!(feed.new_items().len(), 0);
        assert_eq!(feed.archive_items().len(), 2);
    }

    #[test]
    fn feed_items_newest_first() {
        let mut feed = VineFeed::new();
        feed.handle_event(VineEvent::FollowCreator { address: addr(0xAA) });
        for (ts, seed) in [(100, 1), (300, 2), (200, 3)] {
            feed.handle_event(VineEvent::VineAnnounced {
                item: make_item(addr(0xAA), ts, seed),
            });
        }
        let items = feed.new_items();
        assert_eq!(items[0].timestamp, 300);
        assert_eq!(items[1].timestamp, 200);
        assert_eq!(items[2].timestamp, 100);
    }

    #[test]
    fn duplicate_follow_is_idempotent() {
        let mut feed = VineFeed::new();
        feed.handle_event(VineEvent::FollowCreator { address: addr(0xAA) });
        let actions = feed.handle_event(VineEvent::FollowCreator { address: addr(0xAA) });
        // Should still emit subscribe (idempotent), not error
        assert!(!actions.is_empty());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd /Users/zeblith/work/zeblithic/harmony && cargo test -p harmony-browser vine`
Expected: FAIL -- module doesn't exist

**Step 3: Write minimal implementation**

```rust
// crates/harmony-browser/src/vine.rs
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use hashbrown::HashSet;

/// A single vine in the feed.
#[derive(Debug, Clone, PartialEq)]
pub struct VineFeedItem {
    pub bundle_cid: [u8; 32],
    pub video_cid: [u8; 32],
    pub creator: [u8; 16],
    pub timestamp: u64,
    pub title: Option<String>,
    pub reshare_of: Option<[u8; 32]>,
}

/// Events the VineFeed state machine accepts.
#[derive(Debug, Clone)]
pub enum VineEvent {
    FollowCreator { address: [u8; 16] },
    UnfollowCreator { address: [u8; 16] },
    VineAnnounced { item: VineFeedItem },
    MarkViewed { bundle_cid: [u8; 32] },
    MarkAllViewed,
}

/// Actions the VineFeed state machine emits.
#[derive(Debug, Clone)]
pub enum VineAction {
    Subscribe { key_expr: String },
    Unsubscribe { key_expr: String },
    FeedUpdated,
}

/// Sans-I/O state machine for vine feed management.
pub struct VineFeed {
    followed: HashSet<[u8; 16]>,
    items: BTreeMap<(u64, [u8; 32]), VineFeedItem>,
    viewed: HashSet<[u8; 32]>,
}

impl VineFeed {
    pub fn new() -> Self {
        Self {
            followed: HashSet::new(),
            items: BTreeMap::new(),
            viewed: HashSet::new(),
        }
    }

    pub fn handle_event(&mut self, event: VineEvent) -> Vec<VineAction> {
        match event {
            VineEvent::FollowCreator { address } => {
                self.followed.insert(address);
                let addr_hex = hex::encode(address);
                vec![VineAction::Subscribe {
                    key_expr: alloc::format!("harmony/vines/{}/announce/**", addr_hex),
                }]
            }
            VineEvent::UnfollowCreator { address } => {
                if self.followed.remove(&address) {
                    let addr_hex = hex::encode(address);
                    self.items.retain(|_, item| item.creator != address);
                    vec![
                        VineAction::Unsubscribe {
                            key_expr: alloc::format!("harmony/vines/{}/announce/**", addr_hex),
                        },
                        VineAction::FeedUpdated,
                    ]
                } else {
                    vec![]
                }
            }
            VineEvent::VineAnnounced { item } => {
                if !self.followed.contains(&item.creator) {
                    return vec![];
                }
                let key = (item.timestamp, item.bundle_cid);
                self.items.insert(key, item);
                vec![VineAction::FeedUpdated]
            }
            VineEvent::MarkViewed { bundle_cid } => {
                self.viewed.insert(bundle_cid);
                vec![VineAction::FeedUpdated]
            }
            VineEvent::MarkAllViewed => {
                for (_, item) in &self.items {
                    self.viewed.insert(item.bundle_cid);
                }
                vec![VineAction::FeedUpdated]
            }
        }
    }

    /// Unviewed items, newest first.
    pub fn new_items(&self) -> Vec<&VineFeedItem> {
        self.items.values().rev()
            .filter(|item| !self.viewed.contains(&item.bundle_cid))
            .collect()
    }

    /// All items, newest first.
    pub fn archive_items(&self) -> Vec<&VineFeedItem> {
        self.items.values().rev().collect()
    }

    pub fn is_followed(&self, address: &[u8; 16]) -> bool {
        self.followed.contains(address)
    }
}
```

Add to `crates/harmony-browser/src/lib.rs`:
```rust
pub mod vine;
pub use vine::*;
```

Check `crates/harmony-browser/Cargo.toml` -- add `hex` if not already a dependency:
```toml
hex = { version = "0.4", default-features = false, features = ["alloc"] }
```

**Step 4: Run test to verify it passes**

Run: `cd /Users/zeblith/work/zeblithic/harmony && cargo test -p harmony-browser`
Expected: PASS -- all existing + 9 new vine tests

**Step 5: Run full workspace**

Run: `cd /Users/zeblith/work/zeblithic/harmony && cargo test --workspace && cargo clippy --workspace`
Expected: PASS, zero warnings

**Step 6: Commit**

```bash
cd /Users/zeblith/work/zeblithic/harmony
git add crates/harmony-browser/src/vine.rs crates/harmony-browser/src/lib.rs crates/harmony-browser/Cargo.toml
git commit -m "feat(browser): add VineFeed sans-I/O state machine"
```

---

## Phase 2: Browser Integration (harmony-browser repo)

**Prerequisite:** Phase 1 must be merged or the harmony git dependency updated to the Phase 1 branch. For development, consider using path dependencies:
```toml
# In src-tauri/Cargo.toml (temporary, for development)
harmony-browser = { path = "../../harmony/crates/harmony-browser" }
harmony-content = { path = "../../harmony/crates/harmony-content" }
```

### Task 5: Tauri Vine Commands and Fixture Data

**Files:**
- Modify: `src-tauri/src/lib.rs` (add vine commands, update ActionResponse, `mime_to_string`)
- Create: `src-tauri/src/vine_fixtures.rs`
- Modify: `src-tauri/Cargo.toml` (add `base64` dep)

**Context:** Wire VineFeed state machine into the Tauri shell. Add fixture vine data for demo. The Tauri app manages `Mutex<VineFeed>` alongside existing `Mutex<BrowserCore>`. Video data is base64-encoded for IPC to the frontend.

**Step 1: Update ActionResponse and mime handling**

In `src-tauri/src/lib.rs`:

Add `content_base64` field to `ActionResponse`:
```rust
#[derive(Debug, Clone, Serialize)]
pub struct ActionResponse {
    cid: String,
    mime: String,
    content_html: String,
    content_base64: Option<String>,  // base64-encoded binary (video, images)
    trust_level: String,
}
```

Add to `mime_to_string`:
```rust
MimeHint::Video => "video".into(),
```

Update `resolve_render_action` to handle `MimeHint::Video`:
```rust
MimeHint::Video => {
    content_html = String::new();
    content_base64 = Some(base64::engine::general_purpose::STANDARD.encode(&data));
}
```

Update all existing `ActionResponse` construction sites to include `content_base64: None`.

**Step 2: Create vine fixture data**

```rust
// src-tauri/src/vine_fixtures.rs
use harmony_content::blob::{BlobStore, MemoryBlobStore};
use harmony_content::bundle::BundleBuilder;
use harmony_content::cid::ContentId;
use harmony_content::vine::{VineDescriptor, MIME_VINE_VIDEO};

pub struct VineFixture {
    pub bundle_cid: ContentId,
    pub bundle_data: Vec<u8>,
    pub video_cid: ContentId,
    pub video_data: Vec<u8>,
    pub descriptor: VineDescriptor,
}

pub fn demo_vines() -> Vec<VineFixture> {
    vec![
        build_vine_fixture([0xAA; 16], 1_700_000_000, Some("dancing cat"), &fake_video(1)),
        build_vine_fixture([0xAA; 16], 1_700_000_060, Some("sunset timelapse"), &fake_video(2)),
        build_vine_fixture([0xBB; 16], 1_700_000_030, None, &fake_video(3)),
    ]
}

fn fake_video(seed: u8) -> Vec<u8> {
    // Placeholder bytes -- not a real video. Real playback tested in frontend.
    let mut data = vec![0u8; 1024];
    data[0..4].copy_from_slice(b"VINE");
    data[4] = seed;
    data
}

fn build_vine_fixture(
    creator: [u8; 16],
    timestamp: u64,
    title: Option<&str>,
    video_bytes: &[u8],
) -> VineFixture {
    let mut store = MemoryBlobStore::new();
    let video_cid = store.insert(video_bytes).unwrap();

    let descriptor = VineDescriptor {
        creator_address: creator,
        created_at: timestamp,
        title: title.map(Into::into),
        reshare_of: None,
    };
    let desc_bytes = descriptor.to_bytes();
    let desc_cid = store.insert(&desc_bytes).unwrap();

    let mut builder = BundleBuilder::new();
    builder.add(video_cid);
    builder.add(desc_cid);
    builder.with_metadata(video_bytes.len() as u64, 1, timestamp, MIME_VINE_VIDEO);
    let (bundle_data, bundle_cid) = builder.build().unwrap();

    VineFixture { bundle_cid, bundle_data, video_cid, video_data: video_bytes.to_vec(), descriptor }
}
```

**Step 3: Add Tauri commands**

```rust
// In src-tauri/src/lib.rs

use harmony_browser::{VineFeed, VineEvent, VineFeedItem as CoreVineFeedItem};

#[derive(Debug, Clone, Serialize)]
pub struct VineFeedResponse {
    bundle_cid: String,
    video_cid: String,
    creator: String,
    timestamp: u64,
    title: Option<String>,
    reshare_of: Option<String>,
    viewed: bool,
}

#[tauri::command]
fn get_vine_feed(
    feed_state: State<'_, Mutex<VineFeed>>,
    mode: String,
) -> Result<Vec<VineFeedResponse>, String> {
    let feed = feed_state.lock().map_err(|e| format!("Lock: {e}"))?;
    let items = match mode.as_str() {
        "new" => feed.new_items(),
        "archive" => feed.archive_items(),
        _ => return Err("Invalid mode: use 'new' or 'archive'".into()),
    };
    Ok(items.into_iter().map(|item| VineFeedResponse {
        bundle_cid: hex::encode(item.bundle_cid),
        video_cid: hex::encode(item.video_cid),
        creator: hex::encode(item.creator),
        timestamp: item.timestamp,
        title: item.title.clone(),
        reshare_of: item.reshare_of.map(hex::encode),
        viewed: false,
    }).collect())
}

#[tauri::command]
fn get_vine_video(cid_hex: String) -> Result<String, String> {
    let fixtures = vine_fixtures::demo_vines();
    for fixture in &fixtures {
        if hex::encode(fixture.video_cid.to_bytes()) == cid_hex {
            return Ok(base64::engine::general_purpose::STANDARD.encode(&fixture.video_data));
        }
    }
    Err(format!("Video not found: {}", cid_hex))
}

#[tauri::command]
fn follow_creator(
    feed_state: State<'_, Mutex<VineFeed>>,
    address_hex: String,
) -> Result<(), String> {
    let bytes = hex::decode(&address_hex).map_err(|e| format!("Hex: {e}"))?;
    if bytes.len() != 16 { return Err("Address must be 16 bytes".into()); }
    let mut addr = [0u8; 16];
    addr.copy_from_slice(&bytes);

    let mut feed = feed_state.lock().map_err(|e| format!("Lock: {e}"))?;
    feed.handle_event(VineEvent::FollowCreator { address: addr });

    // Seed with fixture vines from this creator
    let fixtures = vine_fixtures::demo_vines();
    for fixture in fixtures {
        if fixture.descriptor.creator_address == addr {
            feed.handle_event(VineEvent::VineAnnounced {
                item: CoreVineFeedItem {
                    bundle_cid: fixture.bundle_cid.to_bytes(),
                    video_cid: fixture.video_cid.to_bytes(),
                    creator: addr,
                    timestamp: fixture.descriptor.created_at,
                    title: fixture.descriptor.title,
                    reshare_of: None,
                },
            });
        }
    }
    Ok(())
}

#[tauri::command]
fn unfollow_creator(
    feed_state: State<'_, Mutex<VineFeed>>,
    address_hex: String,
) -> Result<(), String> {
    let bytes = hex::decode(&address_hex).map_err(|e| format!("Hex: {e}"))?;
    if bytes.len() != 16 { return Err("Address must be 16 bytes".into()); }
    let mut addr = [0u8; 16];
    addr.copy_from_slice(&bytes);
    let mut feed = feed_state.lock().map_err(|e| format!("Lock: {e}"))?;
    feed.handle_event(VineEvent::UnfollowCreator { address: addr });
    Ok(())
}

#[tauri::command]
fn mark_vine_viewed(
    feed_state: State<'_, Mutex<VineFeed>>,
    cid_hex: String,
) -> Result<(), String> {
    let bytes = hex::decode(&cid_hex).map_err(|e| format!("Hex: {e}"))?;
    if bytes.len() != 32 { return Err("CID must be 32 bytes".into()); }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    let mut feed = feed_state.lock().map_err(|e| format!("Lock: {e}"))?;
    feed.handle_event(VineEvent::MarkViewed { bundle_cid: arr });
    Ok(())
}

#[tauri::command]
fn mark_all_viewed(
    feed_state: State<'_, Mutex<VineFeed>>,
) -> Result<(), String> {
    let mut feed = feed_state.lock().map_err(|e| format!("Lock: {e}"))?;
    feed.handle_event(VineEvent::MarkAllViewed);
    Ok(())
}
```

Update `run()`:
```rust
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(BrowserCore::new()))
        .manage(Mutex::new(VineFeed::new()))
        .invoke_handler(tauri::generate_handler![
            navigate, approve_content,
            get_vine_feed, get_vine_video,
            follow_creator, unfollow_creator,
            mark_vine_viewed, mark_all_viewed,
        ])
        .run(tauri::generate_context!())
        .expect("error while running harmony browser");
}
```

Add to `src-tauri/Cargo.toml`:
```toml
base64 = "0.22"
```

**Step 4: Write and run Rust tests**

```rust
#[cfg(test)]
mod vine_tests {
    use super::*;

    #[test]
    fn vine_fixtures_build_successfully() {
        let fixtures = vine_fixtures::demo_vines();
        assert_eq!(fixtures.len(), 3);
        for f in &fixtures {
            assert_ne!(f.bundle_cid.to_bytes(), [0u8; 32]);
            assert!(!f.video_data.is_empty());
        }
    }

    #[test]
    fn vine_fixtures_unique_cids() {
        let fixtures = vine_fixtures::demo_vines();
        let cids: Vec<_> = fixtures.iter().map(|f| f.bundle_cid.to_bytes()).collect();
        for i in 0..cids.len() {
            for j in (i+1)..cids.len() {
                assert_ne!(cids[i], cids[j]);
            }
        }
    }
}
```

Run: `cd /Users/zeblith/work/zeblithic/harmony-browser/src-tauri && cargo test`
Expected: PASS

**Step 5: Update frontend types**

Add to `src/lib/types.ts`:
```typescript
export interface VineFeedItem {
  bundle_cid: string;
  video_cid: string;
  creator: string;
  timestamp: number;
  title: string | null;
  reshare_of: string | null;
  viewed: boolean;
}
```

Update `ActionResponse`:
```typescript
export interface ActionResponse {
  cid: string;
  mime: string;
  content_html: string;
  content_base64: string | null;
  trust_level: TrustLevel;
}
```

**Step 6: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/vine_fixtures.rs src-tauri/Cargo.toml src/lib/types.ts
git commit -m "feat: add Tauri vine commands and fixture data"
```

---

### Task 6: VinePlayer Svelte Component

**Files:**
- Create: `src/lib/components/VinePlayer.svelte`
- Create: `src/lib/components/__tests__/VinePlayer.test.ts`

**Context:** Renders a `<video>` from base64 data. Auto-loops, muted by default with tap-to-unmute. Shows loading placeholder when video data hasn't been fetched yet. For fixture data (fake bytes), the video won't actually play, but the component behavior is correct.

**Step 1: Write the failing test**

```typescript
// src/lib/components/__tests__/VinePlayer.test.ts
import { render, screen, fireEvent } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import VinePlayer from '../VinePlayer.svelte';

describe('VinePlayer', () => {
  it('shows loading when no video data', () => {
    render(VinePlayer, { props: { videoBase64: null, videoCid: 'abc' } });
    expect(screen.getByText(/loading/i)).toBeTruthy();
  });

  it('renders video element when data provided', () => {
    const { container } = render(VinePlayer, {
      props: { videoBase64: 'AAAA', videoCid: 'abc' },
    });
    expect(container.querySelector('video')).toBeTruthy();
  });

  it('video is muted by default', () => {
    const { container } = render(VinePlayer, {
      props: { videoBase64: 'AAAA', videoCid: 'abc' },
    });
    const video = container.querySelector('video');
    expect(video?.muted).toBe(true);
  });

  it('video has loop attribute', () => {
    const { container } = render(VinePlayer, {
      props: { videoBase64: 'AAAA', videoCid: 'abc' },
    });
    const video = container.querySelector('video');
    expect(video?.loop).toBe(true);
  });

  it('has mute toggle button', () => {
    render(VinePlayer, { props: { videoBase64: 'AAAA', videoCid: 'abc' } });
    expect(screen.getByRole('button', { name: /unmute|mute/i })).toBeTruthy();
  });
});
```

**Step 2: Run test to verify it fails**

Run: `cd /Users/zeblith/work/zeblithic/harmony-browser && npx vitest run src/lib/components/__tests__/VinePlayer.test.ts`
Expected: FAIL -- component doesn't exist

**Step 3: Write minimal implementation**

```svelte
<!-- src/lib/components/VinePlayer.svelte -->
<script lang="ts">
  let {
    videoBase64,
    videoCid,
  }: {
    videoBase64: string | null;
    videoCid: string;
  } = $props();

  let muted = $state(true);

  let videoUrl = $derived.by(() => {
    if (!videoBase64) return null;
    try {
      const binary = atob(videoBase64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i);
      }
      const blob = new Blob([bytes], { type: 'video/webm' });
      return URL.createObjectURL(blob);
    } catch {
      return null;
    }
  });

  function toggleMute() {
    muted = !muted;
  }
</script>

<div class="vine-player" aria-label="Vine video {videoCid}">
  {#if videoUrl}
    <video
      src={videoUrl}
      loop
      muted={muted}
      autoplay
      playsinline
    >
      <track kind="captions" />
    </video>
    <button class="mute-toggle" onclick={toggleMute} aria-label={muted ? 'Unmute' : 'Mute'}>
      {muted ? 'Unmute' : 'Mute'}
    </button>
  {:else}
    <p class="loading">Loading video...</p>
  {/if}
</div>

<style>
  .vine-player {
    position: relative;
    aspect-ratio: 9 / 16;
    max-height: 400px;
    background: #000;
    border-radius: 8px;
    overflow: hidden;
  }

  video {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .mute-toggle {
    position: absolute;
    bottom: 8px;
    right: 8px;
    background: rgba(0, 0, 0, 0.5);
    border: none;
    border-radius: 4px;
    padding: 4px 8px;
    cursor: pointer;
    font-size: 0.75rem;
    color: white;
  }

  .loading {
    color: #888;
    text-align: center;
    padding-top: 40%;
  }
</style>
```

**Step 4: Run test to verify it passes**

Run: `cd /Users/zeblith/work/zeblithic/harmony-browser && npx vitest run src/lib/components/__tests__/VinePlayer.test.ts`
Expected: PASS (5 tests)

**Step 5: Commit**

```bash
git add src/lib/components/VinePlayer.svelte src/lib/components/__tests__/VinePlayer.test.ts
git commit -m "feat: add VinePlayer component with mute toggle and loading state"
```

---

### Task 7: VineCard Svelte Component

**Files:**
- Create: `src/lib/components/VineCard.svelte`
- Create: `src/lib/components/__tests__/VineCard.test.ts`

**Context:** Displays a single vine in the feed: VinePlayer + creator address (truncated) + title + timestamp + "Mark viewed" button. Calls `onloadvideo` when mounted so the parent can lazy-load video data.

**Step 1: Write the failing test**

```typescript
// src/lib/components/__tests__/VineCard.test.ts
import { render, screen } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import VineCard from '../VineCard.svelte';
import type { VineFeedItem } from '../../types';

const mockItem: VineFeedItem = {
  bundle_cid: 'aabb',
  video_cid: 'ccdd',
  creator: 'aa'.repeat(16),
  timestamp: 1_700_000_000,
  title: 'funny cat',
  reshare_of: null,
  viewed: false,
};

describe('VineCard', () => {
  it('renders the title', () => {
    render(VineCard, {
      props: { item: mockItem, videoBase64: null, onmarkviewed: vi.fn(), onloadvideo: vi.fn() },
    });
    expect(screen.getByText('funny cat')).toBeTruthy();
  });

  it('renders truncated creator address', () => {
    render(VineCard, {
      props: { item: mockItem, videoBase64: null, onmarkviewed: vi.fn(), onloadvideo: vi.fn() },
    });
    expect(screen.getByText(/aaaa/)).toBeTruthy();
  });

  it('has mark-viewed button', () => {
    render(VineCard, {
      props: { item: mockItem, videoBase64: null, onmarkviewed: vi.fn(), onloadvideo: vi.fn() },
    });
    expect(screen.getByRole('button', { name: /viewed/i })).toBeTruthy();
  });

  it('calls onmarkviewed with bundle_cid on click', async () => {
    const onmarkviewed = vi.fn();
    render(VineCard, {
      props: { item: mockItem, videoBase64: null, onmarkviewed, onloadvideo: vi.fn() },
    });
    await screen.getByRole('button', { name: /viewed/i }).click();
    expect(onmarkviewed).toHaveBeenCalledWith('aabb');
  });

  it('shows no title when null', () => {
    const noTitleItem = { ...mockItem, title: null };
    const { container } = render(VineCard, {
      props: { item: noTitleItem, videoBase64: null, onmarkviewed: vi.fn(), onloadvideo: vi.fn() },
    });
    expect(container.querySelector('h3')).toBeNull();
  });
});
```

**Step 2: Run test to verify it fails**

Run: `cd /Users/zeblith/work/zeblithic/harmony-browser && npx vitest run src/lib/components/__tests__/VineCard.test.ts`
Expected: FAIL

**Step 3: Write minimal implementation**

```svelte
<!-- src/lib/components/VineCard.svelte -->
<script lang="ts">
  import VinePlayer from './VinePlayer.svelte';
  import type { VineFeedItem } from '../types';

  let {
    item,
    videoBase64,
    onmarkviewed,
    onloadvideo,
  }: {
    item: VineFeedItem;
    videoBase64: string | null;
    onmarkviewed: (bundleCid: string) => void;
    onloadvideo: (videoCid: string) => void;
  } = $props();

  let truncatedCreator = $derived(item.creator.slice(0, 8) + '...');

  function formatTimestamp(ts: number): string {
    return new Date(ts * 1000).toLocaleString();
  }
</script>

<article class="vine-card">
  <VinePlayer {videoBase64} videoCid={item.video_cid} />
  <div class="meta">
    {#if item.title}
      <h3>{item.title}</h3>
    {/if}
    <p class="creator">{truncatedCreator}</p>
    <time datetime={new Date(item.timestamp * 1000).toISOString()}>
      {formatTimestamp(item.timestamp)}
    </time>
    {#if item.reshare_of}
      <p class="reshare">Reshared</p>
    {/if}
  </div>
  <div class="actions">
    <button onclick={() => onmarkviewed(item.bundle_cid)} aria-label="Mark as viewed">
      Mark viewed
    </button>
  </div>
</article>

<style>
  .vine-card {
    border: 1px solid #e0e0e0;
    border-radius: 12px;
    overflow: hidden;
    max-width: 360px;
  }

  .meta {
    padding: 0.5rem 1rem;
  }

  .meta h3 {
    margin: 0 0 0.25rem;
    font-size: 1rem;
  }

  .creator {
    font-family: monospace;
    font-size: 0.8rem;
    color: #888;
    margin: 0;
  }

  time {
    font-size: 0.75rem;
    color: #aaa;
  }

  .reshare {
    font-size: 0.75rem;
    color: var(--trust-unknown);
  }

  .actions {
    padding: 0.5rem 1rem;
    display: flex;
    gap: 0.5rem;
  }

  .actions button {
    padding: 0.25rem 0.75rem;
    border-radius: 4px;
    border: 1px solid currentColor;
    background: transparent;
    cursor: pointer;
    font-size: 0.8rem;
  }

  @media (prefers-color-scheme: dark) {
    .vine-card {
      border-color: #333;
    }
  }
</style>
```

**Step 4: Run test to verify it passes**

Run: `cd /Users/zeblith/work/zeblithic/harmony-browser && npx vitest run src/lib/components/__tests__/VineCard.test.ts`
Expected: PASS (5 tests)

**Step 5: Commit**

```bash
git add src/lib/components/VineCard.svelte src/lib/components/__tests__/VineCard.test.ts
git commit -m "feat: add VineCard component with metadata and mark-viewed"
```

---

### Task 8: VineFeed Component and App Mode Toggle

**Files:**
- Create: `src/lib/components/VineFeed.svelte`
- Create: `src/lib/components/__tests__/VineFeed.test.ts`
- Modify: `src/App.svelte` (add Browse/Feed mode toggle)
- Modify: `src/lib/browser-service.ts` (add vine service functions)
- Modify: `src/lib/browser-service.test.ts` (add vine service tests)

**Context:** VineFeed is the main feed container with "New"/"Archive" toggle and "all caught up" empty state. App.svelte gets Browse/Feed tabs. browser-service.ts gets vine IPC wrappers. This task wires everything together.

**Step 1: Add vine service functions**

In `src/lib/browser-service.ts`:
```typescript
export async function getVineFeed(mode: 'new' | 'archive'): Promise<VineFeedItem[]> {
  return invoke('get_vine_feed', { mode });
}

export async function getVineVideo(cidHex: string): Promise<string> {
  return invoke('get_vine_video', { cidHex });
}

export async function followCreator(addressHex: string): Promise<void> {
  return invoke('follow_creator', { addressHex });
}

export async function unfollowCreator(addressHex: string): Promise<void> {
  return invoke('unfollow_creator', { addressHex });
}

export async function markVineViewed(cidHex: string): Promise<void> {
  return invoke('mark_vine_viewed', { cidHex });
}

export async function markAllViewed(): Promise<void> {
  return invoke('mark_all_viewed');
}
```

Import `VineFeedItem` from `./types`.

**Step 2: Add vine service tests**

In `src/lib/browser-service.test.ts`:
```typescript
describe('vine service', () => {
  it('calls get_vine_feed with mode', async () => {
    mockInvoke.mockResolvedValue([]);
    const result = await getVineFeed('new');
    expect(mockInvoke).toHaveBeenCalledWith('get_vine_feed', { mode: 'new' });
    expect(result).toEqual([]);
  });

  it('calls follow_creator with address', async () => {
    mockInvoke.mockResolvedValue(undefined);
    await followCreator('aa'.repeat(16));
    expect(mockInvoke).toHaveBeenCalledWith('follow_creator', { addressHex: 'aa'.repeat(16) });
  });
});
```

**Step 3: Write VineFeed test**

```typescript
// src/lib/components/__tests__/VineFeed.test.ts
import { render, screen } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import VineFeed from '../VineFeed.svelte';
import type { VineFeedItem } from '../../types';

const mockItems: VineFeedItem[] = [
  {
    bundle_cid: 'aabb',
    video_cid: 'ccdd',
    creator: 'aa'.repeat(16),
    timestamp: 1_700_000_000,
    title: 'vine one',
    reshare_of: null,
    viewed: false,
  },
];

describe('VineFeed', () => {
  it('shows caught-up message when empty', () => {
    render(VineFeed, { props: { items: [], onmarkviewed: vi.fn(), onloadvideo: vi.fn() } });
    expect(screen.getByText(/caught up/i)).toBeTruthy();
  });

  it('renders vine cards for items', () => {
    render(VineFeed, { props: { items: mockItems, onmarkviewed: vi.fn(), onloadvideo: vi.fn() } });
    expect(screen.getByText('vine one')).toBeTruthy();
  });

  it('caught-up message has accessible role', () => {
    render(VineFeed, { props: { items: [], onmarkviewed: vi.fn(), onloadvideo: vi.fn() } });
    expect(screen.getByRole('status')).toBeTruthy();
  });
});
```

**Step 4: Write VineFeed implementation**

```svelte
<!-- src/lib/components/VineFeed.svelte -->
<script lang="ts">
  import VineCard from './VineCard.svelte';
  import type { VineFeedItem } from '../types';

  let {
    items,
    onmarkviewed,
    onloadvideo,
  }: {
    items: VineFeedItem[];
    onmarkviewed: (bundleCid: string) => void;
    onloadvideo: (videoCid: string) => void;
  } = $props();
</script>

<section class="vine-feed">
  {#if items.length === 0}
    <p class="caught-up" role="status">You're all caught up!</p>
  {:else}
    <div class="feed-list">
      {#each items as item (item.bundle_cid)}
        <VineCard
          {item}
          videoBase64={null}
          {onmarkviewed}
          {onloadvideo}
        />
      {/each}
    </div>
  {/if}
</section>

<style>
  .vine-feed {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }

  .caught-up {
    text-align: center;
    margin-top: 4rem;
    color: var(--trust-unknown);
    font-size: 1.2rem;
  }

  .feed-list {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.5rem;
  }
</style>
```

**Step 5: Add Browse/Feed mode toggle to App.svelte**

Replace `src/App.svelte` content. Key changes:
- Add `mode` state: `'browse' | 'feed'`
- Add `feedItems` state and `feedMode` state (`'new' | 'archive'`)
- Add `loadFeed()`, `handleMarkViewed()`, `handleLoadVideo()` functions
- Add tablist with Browse/Feed buttons
- Conditionally render AddressBar+ContentPane (browse) or VineFeed (feed)
- Add feed sub-toggle for New/Archive within feed mode
- Use `$effect` to load feed when switching to feed mode

The mode toggle should use `role="tablist"` and `role="tab"` with `aria-selected` for accessibility.

**Step 6: Run all tests**

Run: `cd /Users/zeblith/work/zeblithic/harmony-browser && npx vitest run`
Expected: PASS -- all existing + new tests

Run: `cd /Users/zeblith/work/zeblithic/harmony-browser/src-tauri && cargo test`
Expected: PASS

**Step 7: Commit**

```bash
git add src/App.svelte src/lib/browser-service.ts src/lib/browser-service.test.ts \
  src/lib/components/VineFeed.svelte src/lib/components/__tests__/VineFeed.test.ts
git commit -m "feat: add VineFeed component and Browse/Feed mode toggle"
```

---

## Dependency Map

```
Phase 1 (harmony repo):
  Task 1 (VineDescriptor) ----\
  Task 2 (Zenoh keyspace)      >-- independent, can parallelize
  Task 3 (MimeHint::Video) ---/
  Task 4 (VineFeed SM) --------------- independent

Phase 2 (harmony-browser repo):
  Task 5 (Tauri commands) ------------ depends on Phase 1 merged
  Task 6 (VinePlayer) ----\
  Task 7 (VineCard) -------->--------- independent of Task 5
  Task 8 (VineFeed + App) ------------ depends on Tasks 5, 6, 7
```

Tasks 1, 2, 3, 4 can be parallelized. Tasks 6, 7 can be parallelized.

## Demo Flow

After all tasks complete:
1. Launch app -- see Browse/Feed tabs
2. Click Feed -- empty, "You're all caught up!"
3. Follow fixture creator `aa..aa` (via dev console or wired button)
4. Feed shows 2 vines from creator AA with titles
5. Click "Mark viewed" -- item moves from New to Archive
6. Switch Archive tab -- see all items
7. Browse mode still works for wiki content

## What's Deferred

- **Real video data:** Fixtures use placeholder bytes. Real WebM needs MediaRecorder (VineRecorder component).
- **Like/reshare/report:** Deferred until networking layer is ready.
- **Real networking:** Fixture-based. Real Zenoh pub/sub when harmony-node connects.
- **Compilation bundles:** Data model supports it, UI deferred.
- **Persistence:** VineFeed state is in-memory. Storage layer needed for durability.
- **IntersectionObserver:** Lazy video loading deferred to when real video data exists.
- **Web proxy:** Server-side, separate project.
