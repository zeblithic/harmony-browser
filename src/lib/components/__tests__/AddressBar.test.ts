import { render, screen, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import AddressBar from '../AddressBar.svelte';

describe('AddressBar', () => {
  it('renders a text input', () => {
    render(AddressBar, { props: { trustLevel: null, onnavigate: vi.fn() } });
    expect(screen.getByRole('textbox')).toBeTruthy();
  });

  it('has accessible label', () => {
    render(AddressBar, { props: { trustLevel: null, onnavigate: vi.fn() } });
    expect(screen.getByLabelText('Address')).toBeTruthy();
  });

  it('calls onnavigate with input value on Enter', async () => {
    const onnavigate = vi.fn();
    render(AddressBar, { props: { trustLevel: null, onnavigate } });
    const input = screen.getByRole('textbox');
    await fireEvent.input(input, { target: { value: 'wiki/hello' } });
    await fireEvent.keyDown(input, { key: 'Enter' });
    expect(onnavigate).toHaveBeenCalledWith('wiki/hello');
  });

  it('does not call onnavigate on non-Enter keys', async () => {
    const onnavigate = vi.fn();
    render(AddressBar, { props: { trustLevel: null, onnavigate } });
    const input = screen.getByRole('textbox');
    await fireEvent.input(input, { target: { value: 'wiki/hello' } });
    await fireEvent.keyDown(input, { key: 'a' });
    expect(onnavigate).not.toHaveBeenCalled();
  });

  it('does not call onnavigate when input is empty', async () => {
    const onnavigate = vi.fn();
    render(AddressBar, { props: { trustLevel: null, onnavigate } });
    const input = screen.getByRole('textbox');
    await fireEvent.keyDown(input, { key: 'Enter' });
    expect(onnavigate).not.toHaveBeenCalled();
  });

  it('does not call onnavigate for whitespace-only input', async () => {
    const onnavigate = vi.fn();
    render(AddressBar, { props: { trustLevel: null, onnavigate } });
    const input = screen.getByRole('textbox');
    await fireEvent.input(input, { target: { value: '   ' } });
    await fireEvent.keyDown(input, { key: 'Enter' });
    expect(onnavigate).not.toHaveBeenCalled();
  });

  it('disables input when loading', () => {
    render(AddressBar, { props: { trustLevel: null, loading: true, onnavigate: vi.fn() } });
    expect(screen.getByRole('textbox')).toHaveProperty('disabled', true);
  });

  it('does not disable input when not loading', () => {
    render(AddressBar, { props: { trustLevel: null, loading: false, onnavigate: vi.fn() } });
    expect(screen.getByRole('textbox')).toHaveProperty('disabled', false);
  });

  it('shows TrustBadge when trustLevel is provided', () => {
    render(AddressBar, { props: { trustLevel: 'full_trust', onnavigate: vi.fn() } });
    expect(screen.getByRole('img', { name: 'Fully trusted' })).toBeTruthy();
  });

  it('does not show TrustBadge when trustLevel is null', () => {
    render(AddressBar, { props: { trustLevel: null, onnavigate: vi.fn() } });
    expect(screen.queryByRole('img')).toBeNull();
  });
});
