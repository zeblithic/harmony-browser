<script lang="ts">
  let {
    videoBase64,
    videoCid,
  }: {
    videoBase64: string | null;
    videoCid: string;
  } = $props();

  let muted = $state(true);

  let videoUrl = $derived.by(() => {
    if (!videoBase64) return null;
    try {
      const binary = atob(videoBase64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i);
      }
      const blob = new Blob([bytes], { type: 'video/webm' });
      return URL.createObjectURL(blob);
    } catch {
      return null;
    }
  });

  function toggleMute() {
    muted = !muted;
  }
</script>

<div class="vine-player" aria-label="Vine video {videoCid}">
  {#if videoUrl}
    <video
      src={videoUrl}
      loop
      muted={muted}
      autoplay
      playsinline
    >
      <track kind="captions" />
    </video>
    <button class="mute-toggle" onclick={toggleMute} aria-label={muted ? 'Unmute' : 'Mute'}>
      {muted ? 'Unmute' : 'Mute'}
    </button>
  {:else}
    <p class="loading">Loading video...</p>
  {/if}
</div>

<style>
  .vine-player {
    position: relative;
    background: #000;
    border-radius: 4px;
    overflow: hidden;
    aspect-ratio: 9 / 16;
    max-height: 400px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  video {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .mute-toggle {
    position: absolute;
    bottom: 8px;
    right: 8px;
    background: rgba(0, 0, 0, 0.6);
    color: #fff;
    border: none;
    border-radius: 4px;
    padding: 4px 8px;
    font-size: 12px;
    cursor: pointer;
  }

  .loading {
    color: #888;
    font-family: monospace;
  }
</style>
