import { render, screen, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import VineCard from '../VineCard.svelte';
import type { VineFeedItem } from '../../types';

function makeItem(overrides: Partial<VineFeedItem> = {}): VineFeedItem {
  return {
    bundle_cid: 'aabb',
    video_cid: 'ccdd',
    creator: 'aa'.repeat(16),
    timestamp: 1700000000,
    title: null,
    reshare_of: null,
    viewed: false,
    ...overrides,
  };
}

describe('VineCard', () => {
  it('shows creator address prefix', () => {
    render(VineCard, { props: { item: makeItem(), onplay: vi.fn() } });
    expect(screen.getByText(/aaaaaaaa\.\.\./)).toBeTruthy();
  });

  it('shows title when present', () => {
    render(VineCard, {
      props: { item: makeItem({ title: 'dancing cat' }), onplay: vi.fn() },
    });
    expect(screen.getByText('dancing cat')).toBeTruthy();
  });

  it('shows reshare badge when reshare_of is set', () => {
    render(VineCard, {
      props: { item: makeItem({ reshare_of: 'ff'.repeat(32) }), onplay: vi.fn() },
    });
    expect(screen.getByText('reshared')).toBeTruthy();
  });

  it('shows new badge when unviewed', () => {
    render(VineCard, {
      props: { item: makeItem({ viewed: false }), onplay: vi.fn() },
    });
    expect(screen.getByText('new')).toBeTruthy();
  });

  it('calls onplay when clicked', async () => {
    const onplay = vi.fn();
    const item = makeItem();
    render(VineCard, { props: { item, onplay } });
    const card = screen.getByRole('button');
    await fireEvent.click(card);
    expect(onplay).toHaveBeenCalledWith(item);
  });

  it('calls onplay on Enter key', async () => {
    const onplay = vi.fn();
    render(VineCard, { props: { item: makeItem(), onplay } });
    const card = screen.getByRole('button');
    await fireEvent.keyDown(card, { key: 'Enter' });
    expect(onplay).toHaveBeenCalled();
  });
});
