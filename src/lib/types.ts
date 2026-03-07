export type TrustLevel = 'full_trust' | 'preview' | 'untrusted' | 'unknown';

export interface ActionResponse {
  cid: string;
  mime: string;
  content_html: string;
  trust_level: TrustLevel;
}
