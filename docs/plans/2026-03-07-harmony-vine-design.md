# Harmony Vine Design

**Goal:** A decentralized micro-video platform built as a native feature of harmony-browser, proving out the content-addressed stack with a concrete, user-facing application.

**Philosophy:** The anti-TikTok. A finite, chronological feed of 6-second videos from people you follow. You catch up, you're done, go touch grass. No algorithm, no infinite scroll, no ad-supported attention harvesting. Like the original Vine — wholesome, creative, fulfilling.

**Architecture:** Vine is a content type (`vine/vid`) in `harmony-content`, with feed aggregation, creation, and playback integrated directly into harmony-browser. The feed is a local aggregation of Zenoh subscriptions to followed creators. No servers, no databases, no centralized infrastructure.

---

## 1. Content Model

### 1.1 Vine Blob

A single vine is one `CidType::Blob` (~1 MB):

- **Encoding:** 720p, 6 seconds, targeting ~1.3 Mbps. MVP accepts webview-native codec output (WebM/VP9 or MP4/H.264 via `MediaRecorder` with `videoBitsPerSecond: 1_300_000`). Future: AV1 via `rav1e` for consistent blob sizes.
- **MIME type:** `vine/vid` (8-byte tag fitting InlineMetadata's MIME field)
- **Identity:** `ContentId` = SHA-256 of raw encoded bytes, truncated to 28 bytes + 4-byte size/tag
- **Immutable:** Same vine always produces the same CID. Re-uploading deduplicates naturally.

The 1 vine = 1 blob mapping is architecturally significant. No chunking needed for individual vines. Each vine is an atomic, content-addressed unit.

### 1.2 Vine Metadata Bundle

A published vine is a `CidType::Bundle(1)` containing:

1. **InlineMetadata CID** — total size, chunk count (1), timestamp, MIME `vine/vid`
2. **Vine blob CID** — the actual video data
3. **Descriptor blob CID** — small msgpack blob:
   - `creator_address: [u8; 16]` — publisher's Harmony address
   - `created_at: u64` — Unix timestamp
   - `title: Option<String>` — optional short title (max 140 chars)
   - `reshare_of: Option<ContentId>` — if reshare, CID of original vine bundle

The bundle CID is the canonical identity of the vine-as-published-work. Two reshares of the same video produce different bundle CIDs but reference the same blob CID — the network stores the video only once.

### 1.3 Compilation Bundle

A compilation is a `CidType::Bundle(2)`:

1. **InlineMetadata CID** — total size (sum of vines), count, timestamp, MIME `vine/cmp`
2. **Vine bundle CIDs** — ordered list, played sequentially

Compilations are nearly free because they're just a list of 32-byte CID references to content that already exists and is already cached across the network. A viewer who's watched 3 of 10 vines in a compilation already has 30% cached.

### 1.4 Reshare

A reshare is a vine metadata bundle where `reshare_of = Some(original_cid)`. The resharer publishes their own bundle (their address, their timestamp) pointing to the original vine's blob CID. No data duplication.

### 1.5 Content Type Structs

Vine-specific content types are defined as well-defined structs in `harmony-content` (not ad-hoc browser logic), making them reusable across any Harmony tool:

- `VineDescriptor` — the descriptor blob schema
- `VineBundle` — typed wrapper around bundle construction/parsing
- `CompilationBundle` — typed wrapper for compilation construction/parsing
- `VineReaction` — like/report payload schema

---

## 2. Feed Architecture

The feed is not an algorithm. It's a finite, chronological list of vines from people you follow.

### 2.1 Follow = Zenoh Subscription

Following creator `address_abc` subscribes to:

```
harmony/vines/{address_hex}/announce/**
```

New vine publications produce Zenoh publications on this key expression. The payload is the vine bundle CID + descriptor. BrowserCore's existing `SubscriptionUpdate` handling aggregates these into the feed.

### 2.2 Feed State

```rust
struct VineFeed {
    followed: HashSet<[u8; 16]>,                       // persisted
    items: BTreeMap<(u64, ContentId), VineFeedItem>,    // (timestamp, bundle_cid)
    viewed: HashSet<ContentId>,                          // persisted
}

struct VineFeedItem {
    bundle_cid: ContentId,
    video_cid: ContentId,
    creator: [u8; 16],
    timestamp: u64,
    title: Option<String>,
    reshare_of: Option<ContentId>,
    liked: bool,
    trust_level: TrustDecision,
}
```

### 2.3 Feed Behavior

- **Two views of the same data:**
  - **"New"** — unread items (`!viewed.contains(cid)`), newest first. The active feed.
  - **"Archive"** — all items, chronological, scrollable. For revisiting treasured content. Items are never deleted.
- **Mark as viewed** — explicit user action (per-item or "mark all viewed"). Moves from New to Archive.
- **Finite by design.** When unread is empty: "You're all caught up." No algorithmic padding.
- **Trust-gated.** Reshares from followed creators may reference vines by unknown authors. Those render with the trust badge and gate, same as ContentPane's existing behavior.
- **Offline resilience.** Feed state is local. Offline = see what you have. Reconnect = new SubscriptionUpdate events fill the gap.

### 2.4 Likes and Reports

- **Like:** Msgpack blob `{action: "like", target: bundle_cid, author: address}` published to `harmony/vines/{target_creator}/reactions/{liker_address}`. Public signal, not stored centrally.
- **Report:** Msgpack blob `{action: "report", target: bundle_cid, reason: "..."}` published to `harmony/moderation/{community}/reports/**`. Goes to community moderation keyspace, not to the creator.

---

## 3. Shareable Links

### 3.1 Harmony-Native Links

Canonical reference: `hmy:{64_hex_chars}` (the vine bundle CID). Any Harmony client resolves directly.

Human-friendly names via Zenoh queryables:

```
harmony/vines/bestof/funny-cat  ->  resolves to bundle CID
```

### 3.2 Web-Proxied Links

Thin HTTP reverse proxy on a willing domain (e.g., `q8.fyi`):

```
https://q8.fyi/v/{cid_hex}
```

The proxy:
1. Receives request
2. Resolves CID via its local Harmony node
3. Streams vine as standard HTML page with `<video>` tag
4. Includes banner: "This content lives on the Harmony network"

Read-only bridge. The proxy never hosts content — it fetches from the network on demand. Any willing domain operator can run one. Content remains content-addressed and decentralized.

---

## 4. Creation Flow

Three actions: record, preview, publish.

### 4.1 Recording

- **MVP:** Use the webview's `MediaRecorder` API in the Svelte frontend. `getUserMedia` gives camera access, `MediaRecorder` captures video. No Tauri backend camera integration needed.
- **Hard limit:** 6 seconds. Recording auto-stops. No configuration.
- **Alternative input:** File picker for uploading a pre-existing clip, trimmed to 6 seconds.
- **Codec:** Constrain `MediaRecorder` to VP9/WebM with `videoBitsPerSecond: 1_300_000`. Most browsers support this.

### 4.2 Publishing Pipeline

1. Frontend captures 6-second video, sends bytes to Tauri backend via IPC
2. Backend creates:
   - Vine blob: `BlobStore::insert(video_bytes)` -> `video_cid`
   - Descriptor blob: msgpack(creator, timestamp, title, reshare_of) -> `descriptor_cid`
   - Vine bundle: `BundleBuilder` with metadata + video CID + descriptor CID -> `(bundle_data, bundle_cid)`
3. Backend announces via Zenoh publication on `harmony/vines/{creator_address}/announce/{bundle_cid_hex}`
4. Returns `bundle_cid` to frontend for confirmation

### 4.3 Resharing

1. User clicks "Reshare" on a vine
2. Backend creates new descriptor with `reshare_of = Some(original_bundle_cid)`, resharer's address/timestamp
3. New bundle references same video blob CID — no data duplication
4. Announces on resharer's key expression

### 4.4 Creating Compilations

1. User selects multiple vines (from feed, archive, or own vines)
2. Backend creates compilation bundle: ordered vine bundle CIDs + compilation metadata
3. Announces on `harmony/vines/{creator_address}/compilations/{compilation_cid_hex}`

---

## 5. Browser Integration

### 5.1 BrowserCore State Machine Extensions

**New BrowserEvents:**
- `FollowCreator { address }` / `UnfollowCreator { address }`
- `PublishVine { video_bytes, title }`
- `ReshareVine { bundle_cid }`
- `LikeVine { bundle_cid }`
- `ReportVine { bundle_cid, reason }`
- `MarkViewed { bundle_cid }` / `MarkAllViewed`

**New BrowserActions:**
- `PublishContent { key_expr, payload }` — announce via Zenoh
- `VineFeedUpdated { items }` — emit current feed state to UI

Follow/unfollow translates to existing `Subscribe`/unsubscribe actions. No new networking primitives.

### 5.2 Frontend: Two Navigation Modes

1. **Browse mode** (existing) — address bar navigation, renders content by MIME type including vine playback for `vine/vid`
2. **Feed mode** (new) — chronological vine feed from followed creators

Toggle/tab in the UI switches between them.

### 5.3 New Svelte Components

- **`VinePlayer.svelte`** — `<video>` element with vine blob as source via `URL.createObjectURL`. Auto-loops continuously. Muted by default (tap to unmute). Plays only when in viewport (`IntersectionObserver`). Shows loop counter.
- **`VineFeed.svelte`** — main feed container with "New" / "Archive" toggle
- **`VineCard.svelte`** — single feed item: VinePlayer + creator info + trust badge + like/reshare/report/mark-viewed buttons
- **`VineRecorder.svelte`** — camera capture UI with 6-second countdown, preview, title input, publish button

### 5.4 Vine MIME in ContentPane

Navigating to a vine CID via address bar renders it using VinePlayer — same component as the feed, standalone. Shared `hmy:` links to vines "just work."

---

## 6. MVP Summary

| Layer | What We Build |
|-------|--------------|
| Content | `vine/vid` and `vine/cmp` MIME types, VineDescriptor/VineBundle/CompilationBundle structs in `harmony-content` |
| State | VineFeed state, follow/unfollow, mark-viewed, like, report, publish/reshare events in `harmony-browser` |
| Network | Zenoh pub/sub on `harmony/vines/{address}/**` for announcements and reactions |
| UI | Feed mode toggle, VineFeed, VineCard, VinePlayer (auto-loop, muted-default, IntersectionObserver), VineRecorder |
| Sharing | `hmy:` CID links work natively; thin HTTP proxy spec for web bridge |

---

## 7. North Star / Future Work

### Networking & Distribution
- **Application Layer Multicast (ALM):** Broadcast trees for viral vines so the originator's uplink isn't crushed. High-capacity nodes relay to leaf nodes.
- **Media over QUIC (MoQ):** Live streaming scenarios (creator broadcasting a "vine session" in real-time).
- **SCION data plane:** Path-aware routing for video delivery latency optimization at scale.

### Discovery & Search
- **Matryoshka embeddings ("Matrica"):** Semantic vector search — "funny dog" finds dog videos without tags. Truncatable embeddings scale from mobile (64-dim) to desktop (768-dim).
- **Tantivy/Quickwit:** Full-text search over vine titles and creator names. Distributed index splits queried statelessly.
- **Hashtags:** Text tags in the descriptor blob. Low-hanging fruit for the data model; discovery UI is the complex part.

### AI & Moderation
- **Edge AI moderation via Candle:** Community nodes run quantized visual/audio models on 180 frames before caching. Feasible on consumer hardware.
- **Generative AI creation:** Local text-to-video via Candle for creator tools within the browser.

### Social & Economic
- **Comments:** Threaded discussions on vines. Deferred to Discord/Harmony chat.
- **CredRank reputation:** Graph-based reputation, Markov chains. Moderation power flows to valuable contributors.
- **Probabilistic micropayments:** Direct creator compensation.
- **Community federations:** Self-assembled groups with own moderation bylaws and curation.
- **Selective disclosure:** Prove reputation across communities without revealing identity.

### Platform & Runtime
- **HarmonyApp (WASM):** Vine becomes a WASM bundle running in the browser's sandboxed compute engine. Browser stays generic, Vine is "just an app." Requires UCANs and host APIs.
- **Mobile browser:** Vine on mobile — same CIDs, same network. Record on phone, followers see it on desktop instantly.
- **Deterministic transcoding:** `harmony-workflow` + `harmony-compute` for reproducible WASM transcoding to multiple quality levels. Same input = same output CID = cacheable.
