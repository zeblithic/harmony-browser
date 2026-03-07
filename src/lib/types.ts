export interface ActionResponse {
  cid: string;
  mime: string;
  content_html: string;
  trust_level: 'full_trust' | 'preview' | 'untrusted' | 'unknown';
}
