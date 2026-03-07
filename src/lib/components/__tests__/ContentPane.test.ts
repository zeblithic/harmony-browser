import { render, screen } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import ContentPane from '../ContentPane.svelte';
import type { ActionResponse } from '../../types';

describe('ContentPane', () => {
  it('shows empty state when no content', () => {
    render(ContentPane, { props: { content: null, error: null } });
    expect(screen.getByText('Enter an address to browse the Harmony network')).toBeTruthy();
  });

  it('shows error message', () => {
    render(ContentPane, { props: { content: null, error: 'Content not found' } });
    expect(screen.getByRole('alert')).toBeTruthy();
    expect(screen.getByText('Content not found')).toBeTruthy();
  });

  it('renders markdown content as HTML', () => {
    const content: ActionResponse = {
      cid: 'abc123',
      mime: 'markdown',
      content_html: '<h1>Hello World</h1>\n<p>Some content</p>\n',
      trust_level: 'unknown',
    };
    render(ContentPane, { props: { content, error: null } });
    const article = screen.getByRole('article');
    expect(article).toBeTruthy();
    expect(article.innerHTML).toContain('<h1>Hello World</h1>');
  });

  it('renders plain text in a pre element', () => {
    const content: ActionResponse = {
      cid: 'abc123',
      mime: 'plain_text',
      content_html: 'just plain text',
      trust_level: 'unknown',
    };
    render(ContentPane, { props: { content, error: null } });
    const pre = document.querySelector('pre');
    expect(pre).toBeTruthy();
    expect(pre!.textContent).toBe('just plain text');
  });

  it('shows trust-gated placeholder for untrusted content', () => {
    const content: ActionResponse = {
      cid: 'abc123',
      mime: 'markdown',
      content_html: '<p>secret stuff</p>',
      trust_level: 'untrusted',
    };
    render(ContentPane, { props: { content, error: null } });
    expect(screen.getByText(/Content blocked/)).toBeTruthy();
    expect(screen.getByRole('button', { name: /Approve/i })).toBeTruthy();
  });

  it('renders full content for full_trust', () => {
    const content: ActionResponse = {
      cid: 'abc123',
      mime: 'markdown',
      content_html: '<p>trusted content</p>',
      trust_level: 'full_trust',
    };
    render(ContentPane, { props: { content, error: null } });
    const article = screen.getByRole('article');
    expect(article.innerHTML).toContain('<p>trusted content</p>');
  });

  it('renders full content for unknown trust (text is safe)', () => {
    const content: ActionResponse = {
      cid: 'abc123',
      mime: 'markdown',
      content_html: '<p>unknown author content</p>',
      trust_level: 'unknown',
    };
    render(ContentPane, { props: { content, error: null } });
    const article = screen.getByRole('article');
    expect(article.innerHTML).toContain('<p>unknown author content</p>');
  });
});
