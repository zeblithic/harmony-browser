import { render, screen } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import VinePlayer from '../VinePlayer.svelte';

describe('VinePlayer', () => {
  it('shows loading when no video data', () => {
    render(VinePlayer, { props: { videoBase64: null, videoCid: 'abc' } });
    expect(screen.getByText(/loading/i)).toBeTruthy();
  });

  it('renders video element when data provided', () => {
    const { container } = render(VinePlayer, {
      props: { videoBase64: 'AAAA', videoCid: 'abc' },
    });
    expect(container.querySelector('video')).toBeTruthy();
  });

  it('video is muted by default', () => {
    const { container } = render(VinePlayer, {
      props: { videoBase64: 'AAAA', videoCid: 'abc' },
    });
    const video = container.querySelector('video');
    expect(video?.muted).toBe(true);
  });

  it('video has loop attribute', () => {
    const { container } = render(VinePlayer, {
      props: { videoBase64: 'AAAA', videoCid: 'abc' },
    });
    const video = container.querySelector('video');
    expect(video?.loop).toBe(true);
  });

  it('has mute toggle button', () => {
    render(VinePlayer, { props: { videoBase64: 'AAAA', videoCid: 'abc' } });
    expect(screen.getByRole('button', { name: /unmute|mute/i })).toBeTruthy();
  });
});
