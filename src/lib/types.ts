export type TrustLevel = 'full_trust' | 'preview' | 'untrusted' | 'unknown';

export interface ActionResponse {
  cid: string;
  mime: string;
  content_html: string;
  content_base64: string | null;
  trust_level: TrustLevel;
}

export interface VineFeedItem {
  bundle_cid: string;
  video_cid: string;
  creator: string;
  timestamp: number;
  title: string | null;
  reshare_of: string | null;
  viewed: boolean;
}
