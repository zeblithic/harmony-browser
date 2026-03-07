<script lang="ts">
  import type { TrustLevel } from '../types';

  const LABELS: Record<TrustLevel, string> = {
    full_trust: 'Fully trusted',
    preview: 'Preview trust',
    untrusted: 'Untrusted',
    unknown: 'Unknown author',
  };

  const COLORS: Record<TrustLevel, string> = {
    full_trust: 'var(--trust-full)',
    preview: 'var(--trust-preview)',
    untrusted: 'var(--trust-untrusted)',
    unknown: 'var(--trust-unknown)',
  };

  let { level }: { level: TrustLevel } = $props();
</script>

<span
  role="img"
  aria-label={LABELS[level]}
  class="trust-badge"
  style:background-color={COLORS[level]}
></span>

<style>
  .trust-badge {
    display: inline-block;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    flex-shrink: 0;
  }
</style>
