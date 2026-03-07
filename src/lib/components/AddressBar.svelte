<script lang="ts">
  import TrustBadge from './TrustBadge.svelte';
  import type { TrustLevel } from '../types';

  let {
    trustLevel,
    onnavigate,
  }: {
    trustLevel: TrustLevel | null;
    onnavigate: (input: string) => void;
  } = $props();

  let inputValue = $state('');

  function handleKeyDown(e: KeyboardEvent) {
    const trimmed = inputValue.trim();
    if (e.key === 'Enter' && trimmed !== '') {
      onnavigate(trimmed);
    }
  }
</script>

<nav class="address-bar">
  {#if trustLevel}
    <TrustBadge level={trustLevel} />
  {/if}
  <label>
    <span class="sr-only">Address</span>
    <input
      type="text"
      bind:value={inputValue}
      onkeydown={handleKeyDown}
      placeholder="hmy:... or wiki/topic or ~presence/**"
      aria-label="Address"
    />
  </label>
</nav>

<style>
  .address-bar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    border-bottom: 1px solid #e0e0e0;
  }

  label {
    flex: 1;
  }

  input {
    width: 100%;
    padding: 0.5rem;
    font-family: monospace;
    font-size: 0.9rem;
    border: 1px solid #ccc;
    border-radius: 4px;
    background: inherit;
    color: inherit;
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }

  @media (prefers-color-scheme: dark) {
    .address-bar {
      border-bottom-color: #333;
    }

    input {
      border-color: #555;
    }
  }
</style>
