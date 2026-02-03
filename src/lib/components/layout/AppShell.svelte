<script lang="ts">
  import TitleBar from "./TitleBar.svelte";
  import StatusBar from "./StatusBar.svelte";
  import NowPlayingBar from "./NowPlayingBar.svelte";
  import type { Snippet } from "svelte";

  let { children }: { children: Snippet } = $props();
</script>

<div class="flex h-full flex-col">
  <TitleBar />
  <StatusBar />
  <main class="min-h-0 flex-1 overflow-y-auto bg-[var(--color-bg)] pt-[64px]">
    {@render children()}
  </main>
  <NowPlayingBar />
</div>
