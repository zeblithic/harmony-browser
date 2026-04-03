<script lang="ts">
  import type { VineFeedItem } from '../types';
  import VineCard from './VineCard.svelte';
  import VinePlayer from './VinePlayer.svelte';
  import { onMount } from 'svelte';
  import { getVineFeed, getVineVideo, followCreator, markVineViewed, markAllViewed } from '../browser-service';

  let items = $state<VineFeedItem[]>([]);
  let filter = $state<'new' | 'archive'>('new');
  let loading = $state(false);
  let error = $state<string | null>(null);

  let activeItem = $state<VineFeedItem | null>(null);
  let activeVideoBase64 = $state<string | null>(null);

  let unviewedCount = $derived(items.filter(i => !i.viewed).length);

  onMount(() => { loadFeed(); });

  let loadGeneration = 0;

  async function loadFeed() {
    const gen = ++loadGeneration;
    loading = true;
    error = null;
    try {
      const result = await getVineFeed(filter);
      if (gen !== loadGeneration) return; // stale response from a superseded call
      items = result;
    } catch (e) {
      if (gen !== loadGeneration) return;
      error = String(e);
    } finally {
      if (gen === loadGeneration) loading = false;
    }
  }

  async function handlePlay(item: VineFeedItem) {
    activeItem = item;
    activeVideoBase64 = null;
    try {
      activeVideoBase64 = await getVineVideo(item.video_cid);
      await markVineViewed(item.bundle_cid);
      await loadFeed();
    } catch (e) {
      error = String(e);
      closePlayer();
    }
  }

  function closePlayer() {
    activeItem = null;
    activeVideoBase64 = null;
  }

  function handleOverlayKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      closePlayer();
    }
  }

  async function handleFollowDemo() {
    try {
      // Follow both demo creators to seed the feed.
      await followCreator('aa'.repeat(16));
      await followCreator('bb'.repeat(16));
      await loadFeed();
    } catch (e) {
      error = String(e);
    }
  }

  async function handleMarkAllViewed() {
    try {
      await markAllViewed();
      await loadFeed();
    } catch (e) {
      error = String(e);
    }
  }

  function setFilter(f: 'new' | 'archive') {
    filter = f;
    loadFeed();
  }
</script>

<div class="vine-feed">
  <header class="feed-header">
    <h2>
      Vines
      {#if unviewedCount > 0}
        <span class="unviewed-count">{unviewedCount}</span>
      {/if}
    </h2>
    <div class="header-actions">
      <button onclick={handleFollowDemo}>Load Demo</button>
      <button onclick={handleMarkAllViewed}>Mark All Read</button>
    </div>
  </header>

  <div class="filter-tabs" role="tablist">
    <button
      role="tab"
      aria-selected={filter === 'new'}
      class:active={filter === 'new'}
      onclick={() => setFilter('new')}
    >New</button>
    <button
      role="tab"
      aria-selected={filter === 'archive'}
      class:active={filter === 'archive'}
      onclick={() => setFilter('archive')}
    >All</button>
  </div>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  {#if loading}
    <p class="status">Loading...</p>
  {:else if items.length === 0}
    <p class="status">
      {filter === 'new' ? 'No new vines. Click "Load Demo" to get started.' : 'No vines yet.'}
    </p>
  {:else}
    <div class="feed-list">
      {#each items as item (item.bundle_cid)}
        <VineCard {item} onplay={handlePlay} />
      {/each}
    </div>
  {/if}

  {#if activeItem}
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <div class="player-overlay" role="dialog" aria-label="Vine player" tabindex="0" onkeydown={handleOverlayKeydown}>
      <div class="player-container">
        <button class="close-btn" onclick={closePlayer} aria-label="Close player">X</button>
        {#if activeItem.title}
          <p class="player-title">{activeItem.title}</p>
        {/if}
        <VinePlayer videoBase64={activeVideoBase64} videoCid={activeItem.video_cid} />
      </div>
    </div>
  {/if}
</div>

<style>
  .vine-feed {
    font-family: monospace;
    max-width: 480px;
  }

  .feed-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .feed-header h2 {
    margin: 0;
    font-size: 16px;
    color: #eee;
  }

  .unviewed-count {
    background: #4a9eff;
    color: #000;
    border-radius: 8px;
    padding: 0 6px;
    font-size: 12px;
    margin-left: 6px;
  }

  .header-actions {
    display: flex;
    gap: 6px;
  }

  .header-actions button {
    font-family: monospace;
    font-size: 11px;
    background: #222;
    color: #aaa;
    border: 1px solid #444;
    border-radius: 3px;
    padding: 2px 8px;
    cursor: pointer;
  }

  .header-actions button:hover {
    background: #333;
    color: #eee;
  }

  .filter-tabs {
    display: flex;
    gap: 4px;
    margin-bottom: 8px;
  }

  .filter-tabs button {
    font-family: monospace;
    font-size: 12px;
    background: none;
    border: 1px solid #333;
    border-radius: 3px;
    color: #888;
    padding: 2px 12px;
    cursor: pointer;
  }

  .filter-tabs button.active {
    border-color: #4a9eff;
    color: #4a9eff;
  }

  .feed-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .error {
    color: #f66;
    font-size: 12px;
  }

  .status {
    color: #666;
    font-size: 13px;
  }

  .player-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .player-container {
    position: relative;
    max-width: 360px;
    width: 100%;
  }

  .close-btn {
    position: absolute;
    top: -28px;
    right: 0;
    background: none;
    border: none;
    color: #888;
    font-size: 16px;
    cursor: pointer;
    font-family: monospace;
  }

  .close-btn:hover {
    color: #eee;
  }

  .player-title {
    color: #ccc;
    font-size: 13px;
    margin: 0 0 6px;
    text-align: center;
  }
</style>
