<script lang="ts">
  import './app.css';
  import AddressBar from './lib/components/AddressBar.svelte';
  import ContentPane from './lib/components/ContentPane.svelte';
  import VineFeed from './lib/components/VineFeed.svelte';
  import { navigate, approveContent } from './lib/browser-service';
  import type { ActionResponse } from './lib/types';

  type AppMode = 'browse' | 'vines';

  let appMode = $state<AppMode>('browse');
  let content = $state<ActionResponse | null>(null);
  let error = $state<string | null>(null);
  let loading = $state(false);

  async function handleNavigate(input: string) {
    if (loading) return;
    loading = true;
    error = null;
    content = null;
    try {
      content = await navigate(input);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function handleApprove(cidHex: string) {
    if (loading) return;
    loading = true;
    error = null;
    content = null;
    try {
      content = await approveContent(cidHex);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
</script>

<main>
  <nav class="mode-tabs">
    <button class:active={appMode === 'browse'} onclick={() => appMode = 'browse'}>Browse</button>
    <button class:active={appMode === 'vines'} onclick={() => appMode = 'vines'}>Vines</button>
  </nav>

  {#if appMode === 'browse'}
    <AddressBar
      trustLevel={content?.trust_level ?? null}
      {loading}
      onnavigate={handleNavigate}
    />
    <ContentPane {content} {error} {loading} onapprove={handleApprove} />
  {:else}
    <VineFeed />
  {/if}
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  .mode-tabs {
    display: flex;
    gap: 0;
    border-bottom: 1px solid #333;
    padding: 0 8px;
  }

  .mode-tabs button {
    font-family: monospace;
    font-size: 13px;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    color: #666;
    padding: 8px 16px;
    cursor: pointer;
    transition: color 0.15s, border-color 0.15s;
  }

  .mode-tabs button:hover {
    color: #aaa;
  }

  .mode-tabs button.active {
    color: #eee;
    border-bottom-color: #4a9eff;
  }
</style>
