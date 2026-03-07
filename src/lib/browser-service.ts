import { invoke } from '@tauri-apps/api/core';
import type { ActionResponse } from './types';

export async function navigate(input: string): Promise<ActionResponse> {
  return invoke<ActionResponse>('navigate', { input });
}

export async function approveContent(cidHex: string): Promise<ActionResponse> {
  return invoke<ActionResponse>('approve_content', { cidHex });
}
