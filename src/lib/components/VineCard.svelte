<script lang="ts">
  import type { VineFeedItem } from '../types';

  let {
    item,
    onplay,
  }: {
    item: VineFeedItem;
    onplay: (item: VineFeedItem) => void;
  } = $props();

  let shortCreator = $derived(item.creator.slice(0, 8) + '...');

  function formatTime(ts: number): string {
    return new Date(ts * 1000).toLocaleString();
  }

  function handleClick() {
    onplay(item);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      onplay(item);
    }
  }
</script>

<div
  class="vine-card"
  class:unviewed={!item.viewed}
  role="button"
  tabindex="0"
  aria-label="Play vine{item.title ? ': ' + item.title : ''}"
  onclick={handleClick}
  onkeydown={handleKeydown}
>
  <div class="card-header">
    <span class="creator" title={item.creator}>{shortCreator}</span>
    <span class="timestamp">{formatTime(item.timestamp)}</span>
  </div>
  {#if item.title}
    <p class="title">{item.title}</p>
  {/if}
  {#if item.reshare_of}
    <span class="reshare-badge">reshared</span>
  {/if}
  {#if !item.viewed}
    <span class="new-badge">new</span>
  {/if}
</div>

<style>
  .vine-card {
    border: 1px solid #333;
    border-radius: 4px;
    padding: 8px 12px;
    cursor: pointer;
    font-family: monospace;
    font-size: 13px;
    transition: background 0.15s;
  }

  .vine-card:hover, .vine-card:focus-visible {
    background: #1a1a2e;
    outline: 1px solid #555;
  }

  .vine-card.unviewed {
    border-left: 3px solid #4a9eff;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    gap: 8px;
  }

  .creator {
    color: #8af;
  }

  .timestamp {
    color: #666;
    font-size: 11px;
  }

  .title {
    margin: 4px 0 0;
    color: #ccc;
  }

  .reshare-badge {
    display: inline-block;
    margin-top: 4px;
    font-size: 10px;
    color: #a8a;
    border: 1px solid #a8a;
    border-radius: 2px;
    padding: 0 4px;
  }

  .new-badge {
    display: inline-block;
    margin-top: 4px;
    margin-left: 4px;
    font-size: 10px;
    color: #4a9eff;
    border: 1px solid #4a9eff;
    border-radius: 2px;
    padding: 0 4px;
  }
</style>
