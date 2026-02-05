<!-- Static StatusBar variants for showcase. -->
<script lang="ts">
  import {
    ArrowDown,
    Users,
    Play,
    Pause,
    SkipBack,
    SkipForward,
    Subtitles,
    Monitor,
    Unplug,
  } from "lucide-svelte";
</script>

<div class="space-y-3">
  <!-- Idle mode -->
  <div>
    <span class="mb-1 block text-xs font-medium text-[var(--color-text-muted)]">Idle mode</span>
    <div class="overflow-hidden rounded-lg border border-[var(--color-border)]">
      <div
        class="flex h-[26px] items-center justify-between bg-[var(--color-bg-secondary)] px-3 text-xs text-[var(--color-text-muted)]"
      >
        <div class="flex items-center gap-3">
          <span class="flex items-center gap-1">
            <ArrowDown class="h-3 w-3" />
            2.4 MB/s
          </span>
          <span class="flex items-center gap-1">
            <Users class="h-3 w-3" />
            12 connected
          </span>
        </div>
        <div class="flex items-center gap-2">
          <span class="flex items-center gap-1">
            <span class="h-1.5 w-1.5 rounded-full bg-[var(--color-success)]"></span>
            <Monitor class="h-3 w-3" />
            Living Room TV
          </span>
        </div>
      </div>
    </div>
  </div>

  <!-- Idle mode: no device -->
  <div>
    <span class="mb-1 block text-xs font-medium text-[var(--color-text-muted)]">Idle mode (no device)</span>
    <div class="overflow-hidden rounded-lg border border-[var(--color-border)]">
      <div
        class="flex h-[26px] items-center justify-between bg-[var(--color-bg-secondary)] px-3 text-xs text-[var(--color-text-muted)]"
      >
        <div class="flex items-center gap-3">
          <span class="flex items-center gap-1">
            <ArrowDown class="h-3 w-3" />
            0 B/s
          </span>
          <span class="flex items-center gap-1">
            <Users class="h-3 w-3" />
            0 connected
          </span>
        </div>
        <div class="flex items-center gap-2">
          <span class="flex items-center gap-1">
            <span class="h-1.5 w-1.5 rounded-full bg-[var(--color-text-muted)]"></span>
            <Monitor class="h-3 w-3" />
            No cast device
          </span>
        </div>
      </div>
    </div>
  </div>

  <!-- Playing mode -->
  <div>
    <span class="mb-1 block text-xs font-medium text-[var(--color-text-muted)]">Playing mode</span>
    <div class="overflow-hidden rounded-lg border border-[var(--color-border)]">
      <div
        class="flex h-[26px] items-center justify-between bg-[var(--color-bg-secondary)] px-3 text-xs text-[var(--color-text-muted)]"
      >
        <!-- Title -->
        <div class="flex items-center gap-1.5 overflow-hidden">
          <span class="truncate text-xs font-medium text-[var(--color-text)]">
            Big.Buck.Bunny.1080p.mkv
          </span>
        </div>

        <!-- Transport controls -->
        <div class="flex items-center gap-0.5">
          <div class="rounded p-0.5">
            <SkipBack class="h-3 w-3" />
          </div>
          <div
            class="flex h-5 w-5 items-center justify-center rounded-full bg-[var(--color-primary)] text-white"
          >
            <Pause class="h-2.5 w-2.5" />
          </div>
          <div class="rounded p-0.5">
            <SkipForward class="h-3 w-3" />
          </div>
        </div>

        <!-- Right: subtitle, device, disconnect -->
        <div class="flex items-center gap-1.5">
          <div class="flex items-center gap-0.5 rounded px-1 py-0.5 text-[var(--color-primary)]">
            <Subtitles class="h-3 w-3" />
            <span class="max-w-[60px] truncate">English</span>
          </div>

          <div class="flex items-center gap-0.5 rounded px-1 py-0.5">
            <Monitor class="h-3 w-3" />
            <span class="max-w-[80px] truncate">Living Room</span>
          </div>

          <div class="rounded p-0.5">
            <Unplug class="h-3 w-3" />
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- Playing mode: paused, no subtitle -->
  <div>
    <span class="mb-1 block text-xs font-medium text-[var(--color-text-muted)]">Playing mode (paused, no subtitle)</span>
    <div class="overflow-hidden rounded-lg border border-[var(--color-border)]">
      <div
        class="flex h-[26px] items-center justify-between bg-[var(--color-bg-secondary)] px-3 text-xs text-[var(--color-text-muted)]"
      >
        <div class="flex items-center gap-1.5 overflow-hidden">
          <span class="truncate text-xs font-medium text-[var(--color-text)]">
            Sintel.2010.4K.mkv
          </span>
        </div>

        <div class="flex items-center gap-0.5">
          <div class="rounded p-0.5">
            <SkipBack class="h-3 w-3" />
          </div>
          <div
            class="flex h-5 w-5 items-center justify-center rounded-full bg-[var(--color-primary)] text-white"
          >
            <Play class="h-2.5 w-2.5 translate-x-[1px]" />
          </div>
          <div class="rounded p-0.5">
            <SkipForward class="h-3 w-3" />
          </div>
        </div>

        <div class="flex items-center gap-1.5">
          <div class="flex items-center gap-0.5 rounded px-1 py-0.5">
            <Subtitles class="h-3 w-3" />
          </div>
          <div class="flex items-center gap-0.5 rounded px-1 py-0.5">
            <Monitor class="h-3 w-3" />
            <span class="max-w-[80px] truncate">Bedroom</span>
          </div>
          <div class="rounded p-0.5">
            <Unplug class="h-3 w-3" />
          </div>
        </div>
      </div>
    </div>
  </div>
</div>
