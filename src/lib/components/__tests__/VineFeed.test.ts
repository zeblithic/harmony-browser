import { render, screen } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import VineFeed from '../VineFeed.svelte';

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

  it('shows empty state message', () => {
    render(VineFeed);
    expect(screen.getByText(/no new vines/i)).toBeTruthy();
  });

  it('has mark all read button', () => {
    render(VineFeed);
    expect(screen.getByText('Mark All Read')).toBeTruthy();
  });
});
