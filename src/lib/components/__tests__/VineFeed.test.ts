import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import VineFeed from '../VineFeed.svelte';

// Mock the Tauri invoke to prevent real IPC calls during tests.
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

describe('VineFeed', () => {
  it('renders header', () => {
    render(VineFeed);
    expect(screen.getByText('Vines')).toBeTruthy();
  });

  it('shows filter tabs', () => {
    render(VineFeed);
    expect(screen.getByRole('tab', { name: 'New' })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'All' })).toBeTruthy();
  });

  it('shows load demo button', () => {
    render(VineFeed);
    expect(screen.getByText('Load Demo')).toBeTruthy();
  });

  it('shows empty state after mount loads empty feed', async () => {
    render(VineFeed);
    // onMount triggers loadFeed() which resolves to [] from the mock.
    await waitFor(() => {
      expect(screen.getByText(/no new vines/i)).toBeTruthy();
    });
  });

  it('has mark all read button', () => {
    render(VineFeed);
    expect(screen.getByText('Mark All Read')).toBeTruthy();
  });
});
