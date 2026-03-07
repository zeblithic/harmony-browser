<script lang="ts">
  import './app.css';
  import AddressBar from './lib/components/AddressBar.svelte';
  import ContentPane from './lib/components/ContentPane.svelte';
  import { navigate, approveContent } from './lib/browser-service';
  import type { ActionResponse } from './lib/types';

  let content = $state<ActionResponse | null>(null);
  let error = $state<string | null>(null);

  async function handleNavigate(input: string) {
    error = null;
    content = null;
    try {
      content = await navigate(input);
    } catch (e) {
      error = String(e);
    }
  }

  async function handleApprove(cidHex: string) {
    error = null;
    content = null;
    try {
      content = await approveContent(cidHex);
    } catch (e) {
      error = String(e);
    }
  }
</script>

<main>
  <AddressBar
    trustLevel={content?.trust_level ?? null}
    onnavigate={handleNavigate}
  />
  <ContentPane {content} {error} onapprove={handleApprove} />
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
</style>
