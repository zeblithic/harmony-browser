use std::sync::Mutex;

use harmony_browser::{
    BrowserAction, BrowserCore, BrowserEvent, BrowseTarget, MimeHint, ResolvedContent,
    VineEvent, VineFeed, VineFeedItem as CoreVineFeedItem,
};
use base64::Engine;
use serde::Serialize;
use tauri::State;

mod fixtures;
mod vine_fixtures;

#[derive(Debug, Clone, Serialize)]
pub struct ActionResponse {
    cid: String,
    mime: String,
    /// Rendering contract: markdown is ammonia-sanitized HTML (use {@html}).
    /// Plain text is raw UTF-8 (use Svelte text interpolation, NOT {@html}).
    /// Check `mime` to determine the rendering strategy.
    content_html: String,
    /// Base64-encoded binary content (video, images). Only set for non-text MIME types.
    content_base64: Option<String>,
    trust_level: String,
}

fn mime_to_string(mime: &MimeHint) -> String {
    match mime {
        MimeHint::Markdown => "markdown".into(),
        MimeHint::PlainText => "plain_text".into(),
        MimeHint::Image(_) => "image".into(),
        MimeHint::Video => "video".into(),
        MimeHint::Compilation => "compilation".into(),
        MimeHint::HarmonyApp => "harmony_app".into(),
        MimeHint::Unknown(_) => "unknown".into(),
    }
}

fn trust_to_string(trust: &harmony_browser::TrustDecision) -> String {
    match trust {
        harmony_browser::TrustDecision::FullTrust => "full_trust".into(),
        harmony_browser::TrustDecision::Preview => "preview".into(),
        harmony_browser::TrustDecision::Untrusted => "untrusted".into(),
        harmony_browser::TrustDecision::Unknown => "unknown".into(),
    }
}

fn sanitize_html(html: &str) -> String {
    let mut schemes = std::collections::HashSet::new();
    schemes.insert("http");
    schemes.insert("https");
    schemes.insert("mailto");
    schemes.insert("hmy");
    ammonia::Builder::default()
        .url_schemes(schemes)
        .clean(html)
        .to_string()
}

fn render_markdown(md: &str) -> String {
    let parser = pulldown_cmark::Parser::new(md);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    sanitize_html(&html)
}

fn resolve_render_action(action: BrowserAction) -> Option<ActionResponse> {
    match action {
        BrowserAction::Render(ResolvedContent::Static {
            cid, mime, data, trust_level, ..
        }) => {
            let mut content_html = String::new();
            let mut content_base64 = None;
            match mime {
                MimeHint::Markdown => {
                    let text = String::from_utf8_lossy(&data);
                    content_html = render_markdown(&text);
                }
                MimeHint::PlainText => {
                    content_html = String::from_utf8_lossy(&data).into_owned();
                }
                MimeHint::Video | MimeHint::Compilation => {
                    content_base64 =
                        Some(base64::engine::general_purpose::STANDARD.encode(&data));
                }
                _ => {}
            };
            Some(ActionResponse {
                cid: hex::encode(cid.to_bytes()),
                mime: mime_to_string(&mime),
                content_html,
                content_base64,
                trust_level: trust_to_string(&trust_level),
            })
        }
        BrowserAction::Render(_) => {
            eprintln!("Unhandled ResolvedContent variant (Dynamic/future)");
            None
        }
        _ => None,
    }
}

#[tauri::command]
fn navigate(state: State<'_, Mutex<BrowserCore>>, input: String) -> Result<ActionResponse, String> {
    let target = BrowseTarget::parse(&input).map_err(|e| e.to_string())?;
    let mut core = state.lock().map_err(|e| format!("State lock poisoned: {e}"))?;
    let actions = core.handle_event(BrowserEvent::Navigate(target));

    for action in actions {
        match action {
            BrowserAction::FetchContent { cid } => {
                let hex_cid = hex::encode(cid.to_bytes());
                return Err(format!("Direct CID fetch not yet supported: {}", hex_cid));
            }
            BrowserAction::QueryNamed { key_expr } => {
                let fixture = fixtures::resolve_named(&key_expr)
                    .ok_or_else(|| format!("Not found: {}", key_expr))?;

                let render_actions = core.handle_event(BrowserEvent::ContentFetched {
                    cid: fixture.cid,
                    data: fixture.data,
                });

                for ra in render_actions {
                    if let Some(response) = resolve_render_action(ra) {
                        return Ok(response);
                    }
                }
                return Err("Content resolved but could not render".into());
            }
            other => {
                if let Some(response) = resolve_render_action(other) {
                    return Ok(response);
                }
            }
        }
    }

    Err("No actionable result".into())
}

#[tauri::command]
fn approve_content(
    state: State<'_, Mutex<BrowserCore>>,
    cid_hex: String,
) -> Result<ActionResponse, String> {
    let bytes = hex::decode(&cid_hex).map_err(|e| format!("Invalid hex: {}", e))?;
    if bytes.len() != 32 {
        return Err(format!("Expected 32 bytes, got {}", bytes.len()));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    let cid = harmony_content::cid::ContentId::from_bytes(arr);

    let mut core = state.lock().map_err(|e| format!("State lock poisoned: {e}"))?;
    let actions = core.handle_event(BrowserEvent::ApproveContent { cid });

    for action in actions {
        match action {
            BrowserAction::FetchContent { cid } => {
                let fixture = fixtures::resolve_by_cid(&cid)
                    .ok_or_else(|| "Content not found for approved CID".to_string())?;
                let render_actions = core.handle_event(BrowserEvent::ContentFetched {
                    cid: fixture.cid,
                    data: fixture.data,
                });
                for ra in render_actions {
                    if let Some(response) = resolve_render_action(ra) {
                        return Ok(response);
                    }
                }
                return Err("Content resolved but could not render".into());
            }
            other => {
                if let Some(response) = resolve_render_action(other) {
                    return Ok(response);
                }
            }
        }
    }

    Err("Could not resolve approved content".into())
}

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
    let items: Vec<&CoreVineFeedItem> = match mode.as_str() {
        "new" => feed.new_items(),
        "archive" => feed.archive_items(),
        _ => return Err("Invalid mode: use 'new' or 'archive'".into()),
    };
    Ok(items
        .into_iter()
        .map(|item| VineFeedResponse {
            bundle_cid: hex::encode(item.bundle_cid),
            video_cid: hex::encode(item.video_cid),
            creator: hex::encode(item.creator),
            timestamp: item.timestamp,
            title: item.title.clone(),
            reshare_of: item.reshare_of.map(hex::encode),
            viewed: false,
        })
        .collect())
}

#[tauri::command]
fn get_vine_video(cid_hex: String) -> Result<String, String> {
    let fixtures = vine_fixtures::demo_vines();
    for fixture in &fixtures {
        if hex::encode(fixture.video_cid.to_bytes()) == cid_hex {
            return Ok(base64::engine::general_purpose::STANDARD.encode(&fixture.video_data));
        }
    }
    Err(format!("Video not found: {cid_hex}"))
}

#[tauri::command]
fn follow_creator(
    feed_state: State<'_, Mutex<VineFeed>>,
    address_hex: String,
) -> Result<(), String> {
    let bytes = hex::decode(&address_hex).map_err(|e| format!("Hex: {e}"))?;
    if bytes.len() != 16 {
        return Err("Address must be 16 bytes".into());
    }
    let mut addr = [0u8; 16];
    addr.copy_from_slice(&bytes);

    let mut feed = feed_state.lock().map_err(|e| format!("Lock: {e}"))?;
    let _ = feed.handle_event(VineEvent::FollowCreator { address: addr });

    // Seed with fixture vines from this creator.
    for fixture in vine_fixtures::demo_vines() {
        if fixture.descriptor.creator_address == addr {
            let _ = feed.handle_event(VineEvent::VineAnnounced {
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
    if bytes.len() != 16 {
        return Err("Address must be 16 bytes".into());
    }
    let mut addr = [0u8; 16];
    addr.copy_from_slice(&bytes);
    let mut feed = feed_state.lock().map_err(|e| format!("Lock: {e}"))?;
    let _ = feed.handle_event(VineEvent::UnfollowCreator { address: addr });
    Ok(())
}

#[tauri::command]
fn mark_vine_viewed(
    feed_state: State<'_, Mutex<VineFeed>>,
    cid_hex: String,
) -> Result<(), String> {
    let bytes = hex::decode(&cid_hex).map_err(|e| format!("Hex: {e}"))?;
    if bytes.len() != 32 {
        return Err("CID must be 32 bytes".into());
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    let mut feed = feed_state.lock().map_err(|e| format!("Lock: {e}"))?;
    let _ = feed.handle_event(VineEvent::MarkViewed { bundle_cid: arr });
    Ok(())
}

#[tauri::command]
fn mark_all_viewed(feed_state: State<'_, Mutex<VineFeed>>) -> Result<(), String> {
    let mut feed = feed_state.lock().map_err(|e| format!("Lock: {e}"))?;
    let _ = feed.handle_event(VineEvent::MarkAllViewed);
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(BrowserCore::new()))
        .manage(Mutex::new(VineFeed::new()))
        .invoke_handler(tauri::generate_handler![
            navigate,
            approve_content,
            get_vine_feed,
            get_vine_video,
            follow_creator,
            unfollow_creator,
            mark_vine_viewed,
            mark_all_viewed,
        ])
        .run(tauri::generate_context!())
        .expect("error while running harmony browser");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_markdown_basic() {
        let html = render_markdown("# Hello\n\nWorld");
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("<p>World</p>"));
    }

    #[test]
    fn render_markdown_with_bold() {
        let html = render_markdown("This is **bold** text");
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn render_markdown_with_link() {
        let html = render_markdown("[click](hmy:abc123)");
        assert!(html.contains("href=\"hmy:abc123\""));
    }

    #[test]
    fn mime_to_string_values() {
        assert_eq!(mime_to_string(&MimeHint::Markdown), "markdown");
        assert_eq!(mime_to_string(&MimeHint::PlainText), "plain_text");
    }

    #[test]
    fn trust_to_string_values() {
        assert_eq!(trust_to_string(&harmony_browser::TrustDecision::FullTrust), "full_trust");
        assert_eq!(trust_to_string(&harmony_browser::TrustDecision::Unknown), "unknown");
    }

    #[test]
    fn fixtures_resolve_known_paths() {
        assert!(fixtures::resolve_named("harmony/content/wiki/hello").is_some());
        assert!(fixtures::resolve_named("harmony/content/wiki/trust-demo").is_some());
        assert!(fixtures::resolve_named("harmony/content/plain/example").is_some());
    }

    #[test]
    fn fixtures_unknown_path_returns_none() {
        assert!(fixtures::resolve_named("harmony/content/nonexistent").is_none());
    }

    #[test]
    fn render_markdown_strips_raw_html() {
        let html = render_markdown("Hello <script>alert(1)</script> world");
        assert!(!html.contains("<script>"));
        assert!(html.contains("Hello"));
        assert!(html.contains("world"));
    }

    #[test]
    fn render_markdown_strips_event_handlers() {
        let html = render_markdown("<img onerror=\"alert(1)\" src=\"x\">");
        assert!(!html.contains("onerror"));
    }

    #[test]
    fn sanitize_html_strips_scripts() {
        let result = sanitize_html("<p>safe</p><script>alert(1)</script>");
        assert!(result.contains("<p>safe</p>"));
        assert!(!result.contains("<script>"));
    }

    #[test]
    fn fixtures_resolve_by_cid() {
        let fixture = fixtures::resolve_named("harmony/content/wiki/hello").unwrap();
        let found = fixtures::resolve_by_cid(&fixture.cid);
        assert!(found.is_some());
        assert_eq!(found.unwrap().cid, fixture.cid);
    }

    #[test]
    fn fixtures_resolve_by_cid_unknown() {
        let fake_cid = harmony_content::cid::ContentId::from_bytes([0u8; 32]);
        assert!(fixtures::resolve_by_cid(&fake_cid).is_none());
    }
}
