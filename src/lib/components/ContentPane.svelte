<script lang="ts">
  import type { ActionResponse } from '../types';
  import TrustBadge from './TrustBadge.svelte';

  let {
    content,
    error,
    onapprove,
  }: {
    content: ActionResponse | null;
    error: string | null;
    onapprove?: (cidHex: string) => void;
  } = $props();

  function handleApprove() {
    if (content && onapprove) {
      onapprove(content.cid);
    }
  }
</script>

<section class="content-pane">
  {#if error}
    <div role="alert" class="error">{error}</div>
  {:else if !content}
    <p class="empty">Enter an address to browse the Harmony network</p>
  {:else if content.trust_level === 'untrusted'}
    <div class="gated">
      <TrustBadge level={content.trust_level} />
      <p>Content blocked — author untrusted</p>
      <button onclick={handleApprove}>Approve & Load</button>
    </div>
  {:else if content.mime === 'plain_text'}
    <pre>{content.content_html}</pre>
  {:else if content.mime === 'markdown'}
    <article>{@html content.content_html}</article>
  {:else}
    <div class="unsupported">
      <p>Content type <code>{content.mime}</code> is not yet renderable.</p>
    </div>
  {/if}
</section>

<style>
  .content-pane {
    padding: 1rem;
    flex: 1;
    overflow-y: auto;
  }

  .empty {
    color: var(--trust-unknown);
    text-align: center;
    margin-top: 4rem;
  }

  .error {
    color: var(--trust-untrusted);
    padding: 1rem;
    border: 1px solid var(--trust-untrusted);
    border-radius: 4px;
  }

  .gated {
    text-align: center;
    margin-top: 4rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
  }

  .gated button {
    padding: 0.5rem 1rem;
    border-radius: 4px;
    border: 1px solid currentColor;
    background: transparent;
    cursor: pointer;
  }

  article {
    line-height: 1.6;
  }

  pre {
    white-space: pre-wrap;
    word-break: break-word;
    background: #f5f5f5;
    padding: 1rem;
    border-radius: 4px;
  }

  .unsupported {
    text-align: center;
    margin-top: 4rem;
    color: var(--trust-unknown);
  }

  .unsupported code {
    font-size: 0.9rem;
    background: #f0f0f0;
    padding: 0.15rem 0.4rem;
    border-radius: 3px;
  }

  @media (prefers-color-scheme: dark) {
    pre {
      background: #2a2a3e;
    }

    .unsupported code {
      background: #2a2a3e;
    }
  }
</style>
