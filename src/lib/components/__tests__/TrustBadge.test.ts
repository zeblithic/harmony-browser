import { render, screen } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import TrustBadge from '../TrustBadge.svelte';

describe('TrustBadge', () => {
  it('renders green dot for full_trust', () => {
    render(TrustBadge, { props: { level: 'full_trust' } });
    const badge = screen.getByRole('img', { name: 'Fully trusted' });
    expect(badge).toBeTruthy();
    expect(badge.style.backgroundColor).toBe('var(--trust-full)');
  });

  it('renders yellow dot for preview', () => {
    render(TrustBadge, { props: { level: 'preview' } });
    const badge = screen.getByRole('img', { name: 'Preview trust' });
    expect(badge).toBeTruthy();
    expect(badge.style.backgroundColor).toBe('var(--trust-preview)');
  });

  it('renders red dot for untrusted', () => {
    render(TrustBadge, { props: { level: 'untrusted' } });
    const badge = screen.getByRole('img', { name: 'Untrusted' });
    expect(badge).toBeTruthy();
    expect(badge.style.backgroundColor).toBe('var(--trust-untrusted)');
  });

  it('renders gray dot for unknown', () => {
    render(TrustBadge, { props: { level: 'unknown' } });
    const badge = screen.getByRole('img', { name: 'Unknown author' });
    expect(badge).toBeTruthy();
    expect(badge.style.backgroundColor).toBe('var(--trust-unknown)');
  });
});
