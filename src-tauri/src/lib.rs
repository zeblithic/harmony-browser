use std::sync::Mutex;

use harmony_browser::{BrowserAction, BrowserCore, BrowserEvent, BrowseTarget, MimeHint, ResolvedContent};
use serde::Serialize;
use tauri::State;

mod fixtures;

#[derive(Debug, Clone, Serialize)]
pub struct ActionResponse {
    cid: String,
    mime: String,
    content_html: String,
    trust_level: String,
}

fn mime_to_string(mime: &MimeHint) -> String {
    match mime {
        MimeHint::Markdown => "markdown".into(),
        MimeHint::PlainText => "plain_text".into(),
        MimeHint::Image(_) => "image".into(),
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

fn render_markdown(md: &str) -> String {
    let parser = pulldown_cmark::Parser::new(md);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    html
}

fn resolve_render_action(action: BrowserAction) -> Option<ActionResponse> {
    match action {
        BrowserAction::Render(ResolvedContent::Static {
            cid, mime, data, trust_level, ..
        }) => {
            let content_html = match mime {
                MimeHint::Markdown => {
                    let text = String::from_utf8_lossy(&data);
                    render_markdown(&text)
                }
                _ => String::from_utf8_lossy(&data).into_owned(),
            };
            Some(ActionResponse {
                cid: hex::encode(cid.to_bytes()),
                mime: mime_to_string(&mime),
                content_html,
                trust_level: trust_to_string(&trust_level),
            })
        }
        _ => None,
    }
}

#[tauri::command]
fn navigate(state: State<'_, Mutex<BrowserCore>>, input: String) -> Result<ActionResponse, String> {
    let target = BrowseTarget::parse(&input).map_err(|e| e.to_string())?;
    let mut core = state.lock().unwrap();
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
            _ => {}
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

    let mut core = state.lock().unwrap();
    let _actions = core.handle_event(BrowserEvent::ApproveContent { cid });

    Err("Approve noted — re-navigate to see updated trust level".into())
}

pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(BrowserCore::new()))
        .invoke_handler(tauri::generate_handler![navigate, approve_content])
        .run(tauri::generate_context!())
        .expect("error while running harmony browser");
}
