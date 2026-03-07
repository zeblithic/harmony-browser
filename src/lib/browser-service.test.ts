import { describe, it, expect, vi, beforeEach } from 'vitest';
import { navigate, approveContent } from './browser-service';
import type { ActionResponse } from './types';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';
const mockInvoke = vi.mocked(invoke);

describe('browser-service', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  describe('navigate', () => {
    it('calls invoke with navigate command and input', async () => {
      const response: ActionResponse = {
        cid: 'abc123',
        mime: 'markdown',
        content_html: '<p>hello</p>',
        trust_level: 'unknown',
      };
      mockInvoke.mockResolvedValue(response);

      const result = await navigate('wiki/hello');

      expect(mockInvoke).toHaveBeenCalledWith('navigate', { input: 'wiki/hello' });
      expect(result).toEqual(response);
    });

    it('propagates errors from invoke', async () => {
      mockInvoke.mockRejectedValue('Invalid CID');

      await expect(navigate('hmy:bad')).rejects.toBe('Invalid CID');
    });
  });

  describe('approveContent', () => {
    it('calls invoke with approve_content command and cid_hex', async () => {
      const response: ActionResponse = {
        cid: 'abc123',
        mime: 'markdown',
        content_html: '<p>hello</p>',
        trust_level: 'full_trust',
      };
      mockInvoke.mockResolvedValue(response);

      const result = await approveContent('abc123');

      expect(mockInvoke).toHaveBeenCalledWith('approve_content', { cidHex: 'abc123' });
      expect(result).toEqual(response);
    });

    it('propagates errors from invoke', async () => {
      mockInvoke.mockRejectedValue('Could not resolve approved content');
      await expect(approveContent('deadbeef'.repeat(8))).rejects.toBe(
        'Could not resolve approved content'
      );
    });
  });
});
