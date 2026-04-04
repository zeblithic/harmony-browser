import { invoke } from '@tauri-apps/api/core';
import type { ActionResponse, VineFeedItem } from './types';

export async function navigate(input: string): Promise<ActionResponse> {
  return invoke<ActionResponse>('navigate', { input });
}

export async function approveContent(cidHex: string): Promise<ActionResponse> {
  return invoke<ActionResponse>('approve_content', { cidHex });
}

export async function getVineFeed(mode: 'new' | 'archive'): Promise<VineFeedItem[]> {
  return invoke<VineFeedItem[]>('get_vine_feed', { mode });
}

export async function getVineVideo(cidHex: string): Promise<string> {
  return invoke<string>('get_vine_video', { cidHex });
}

export async function followCreator(addressHex: string): Promise<void> {
  return invoke<void>('follow_creator', { addressHex });
}

export async function unfollowCreator(addressHex: string): Promise<void> {
  return invoke<void>('unfollow_creator', { addressHex });
}

export async function markVineViewed(cidHex: string): Promise<void> {
  return invoke<void>('mark_vine_viewed', { cidHex });
}

export async function markAllViewed(): Promise<void> {
  return invoke<void>('mark_all_viewed');
}
