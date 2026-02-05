<!-- Static pipeline cards (When/Where/With/Then) for showcase. -->
<script lang="ts">
  import {
    Link,
    CircleCheck,
    FileSearch,
    ArrowUpDown,
    FolderSearch,
    Filter,
    Files,
    FileVideo,
    Music,
    FileCode,
    Cast,
    FolderOutput,
    Bell,
    MonitorPlay,
    Subtitles,
    Terminal,
    Timer,
    Trash2,
    Plus,
    X,
  } from "lucide-svelte";
</script>

<div class="space-y-3">
  <!-- WHEN card -->
  <div class="rounded-xl border border-[var(--color-primary)]/30 bg-[var(--color-primary)]/5 p-4">
    <div class="flex items-center gap-3">
      <h3 class="text-2xl font-black text-[var(--color-primary)]">When</h3>
      <Link class="h-4 w-4 text-[var(--color-primary)]" />
    </div>

    <div class="mt-3 grid grid-cols-2 gap-2">
      <button class="flex items-center gap-2 rounded-lg bg-[var(--color-primary)] px-3 py-2 text-sm font-medium text-white hover:opacity-80">
        <Link class="h-4 w-4" />
        Torrent added
      </button>
      <button class="flex items-center gap-2 rounded-lg bg-[var(--color-primary)]/15 px-3 py-2 text-sm font-medium text-[var(--color-primary)] hover:opacity-80">
        <CircleCheck class="h-4 w-4" />
        Download complete
      </button>
      <button class="flex items-center gap-2 rounded-lg bg-[var(--color-primary)]/15 px-3 py-2 text-sm font-medium text-[var(--color-primary)] hover:opacity-80">
        <FileSearch class="h-4 w-4" />
        Metadata ready
      </button>
      <button class="flex items-center gap-2 rounded-lg bg-[var(--color-primary)]/15 px-3 py-2 text-sm font-medium text-[var(--color-primary)] hover:opacity-80">
        <ArrowUpDown class="h-4 w-4" />
        Seeding ratio
      </button>
      <button class="flex items-center gap-2 rounded-lg bg-[var(--color-primary)]/15 px-3 py-2 text-sm font-medium text-[var(--color-primary)] hover:opacity-80">
        <FolderSearch class="h-4 w-4" />
        Folder watch
      </button>
    </div>
  </div>

  <!-- Connector -->
  <div class="pointer-events-none flex justify-center">
    <div class="h-4 w-[4px] rounded-full bg-[var(--color-border)]"></div>
  </div>

  <!-- WHERE card -->
  <div class="rounded-xl border border-[var(--color-info)]/30 bg-[var(--color-info)]/5 p-4">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <h3 class="text-2xl font-black text-[var(--color-info)]">Where</h3>
        <Filter class="h-4 w-4 text-[var(--color-info)]" />
        <span class="rounded-full bg-[var(--color-info)]/20 px-2 py-0.5 text-xs font-bold text-[var(--color-info)]">
          3 matches
        </span>
      </div>
      <div class="rounded p-1 text-[var(--color-text-muted)]">
        <Trash2 class="h-4 w-4" />
      </div>
    </div>

    <div class="mt-3 space-y-2">
      <!-- Condition row 1 -->
      <div class="flex items-center gap-2">
        <span class="w-12 shrink-0 text-xs font-bold text-[var(--color-info)]">Name</span>
        <span class="shrink-0 rounded px-1 py-0.5 text-xs font-bold text-[var(--color-text-muted)]">is</span>
        <select class="h-7 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-1.5 text-xs text-[var(--color-text)] outline-none">
          <option>contains</option>
          <option>does not contain</option>
          <option>starts with</option>
          <option>ends with</option>
          <option>equals</option>
        </select>
        <input
          type="text"
          value="1080p"
          readonly
          class="h-7 min-w-0 flex-1 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-xs text-[var(--color-text)] outline-none"
        />
        <div class="shrink-0 rounded p-1 text-[var(--color-text-muted)]">
          <X class="h-3.5 w-3.5" />
        </div>
      </div>
      <!-- Condition row 2 -->
      <div class="flex items-center gap-2">
        <span class="w-12 shrink-0 rounded px-1 py-0.5 text-center text-xs font-bold text-[var(--color-info)]">and</span>
        <span class="shrink-0 rounded px-1 py-0.5 text-xs font-bold text-[var(--color-error)]">not</span>
        <select class="h-7 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-1.5 text-xs text-[var(--color-text)] outline-none">
          <option selected>contains</option>
        </select>
        <input
          type="text"
          value="CAM"
          readonly
          class="h-7 min-w-0 flex-1 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-xs text-[var(--color-text)] outline-none"
        />
        <div class="shrink-0 rounded p-1 text-[var(--color-text-muted)]">
          <X class="h-3.5 w-3.5" />
        </div>
      </div>
    </div>

    <div class="mt-2 flex items-center gap-1.5 rounded-lg px-2 py-1 text-xs text-[var(--color-text-muted)]">
      <Plus class="h-3.5 w-3.5" />
      Add match
    </div>
  </div>

  <!-- Connector -->
  <div class="pointer-events-none flex justify-center">
    <div class="h-4 w-[4px] rounded-full bg-[var(--color-border)]"></div>
  </div>

  <!-- WITH card -->
  <div class="rounded-xl border border-[var(--color-accent)]/30 bg-[var(--color-accent)]/5 p-4">
    <div class="flex items-center gap-3">
      <h3 class="text-2xl font-black text-[var(--color-accent)]">With</h3>
      <FileVideo class="h-4 w-4 text-[var(--color-accent)]" />
    </div>

    <div class="mt-3 grid grid-cols-2 gap-2">
      <button class="flex items-center gap-2 rounded-lg bg-[var(--color-accent)]/15 px-3 py-2 text-sm font-medium text-[var(--color-accent)] hover:opacity-80">
        <Files class="h-4 w-4" />
        Any file
      </button>
      <button class="flex items-center gap-2 rounded-lg bg-[var(--color-accent)] px-3 py-2 text-sm font-medium text-white hover:opacity-80">
        <FileVideo class="h-4 w-4" />
        Video files
      </button>
      <button class="flex items-center gap-2 rounded-lg bg-[var(--color-accent)]/15 px-3 py-2 text-sm font-medium text-[var(--color-accent)] hover:opacity-80">
        <Music class="h-4 w-4" />
        Audio files
      </button>
      <button class="flex items-center gap-2 rounded-lg bg-[var(--color-accent)]/15 px-3 py-2 text-sm font-medium text-[var(--color-accent)] hover:opacity-80">
        <FileCode class="h-4 w-4" />
        Custom
      </button>
    </div>

    <!-- Secondary filters -->
    <div class="mt-3 space-y-2">
      <label class="flex items-center gap-2 text-xs text-[var(--color-text-secondary)]">
        <input type="checkbox" checked class="rounded" />
        Largest file only
      </label>
      <div class="flex items-center gap-2">
        <span class="text-xs text-[var(--color-text-secondary)]">Min size</span>
        <input
          type="number"
          value={100}
          readonly
          class="h-7 w-20 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-xs text-[var(--color-text)] outline-none"
        />
        <span class="text-xs text-[var(--color-text-muted)]">MB</span>
      </div>
    </div>
  </div>

  <!-- Connector -->
  <div class="pointer-events-none flex justify-center">
    <div class="h-4 w-[4px] rounded-full bg-[var(--color-border)]"></div>
  </div>

  <!-- THEN action card (Cast) -->
  <div class="rounded-xl p-4 text-white" style="background-color: var(--color-primary)">
    <div class="mb-3 flex items-center justify-between">
      <div class="flex items-center gap-2">
        <h3 class="text-2xl font-black text-white drop-shadow-sm">Then</h3>
        <Cast class="h-5 w-5 text-white/70" />
      </div>
      <div class="rounded p-1 text-white/40">
        <Trash2 class="h-4 w-4" />
      </div>
    </div>
    <div class="flex items-start gap-2">
      <span class="shrink-0 pt-1.5 text-sm font-semibold drop-shadow-sm">Cast to</span>
      <div class="min-w-0 flex-1">
        <div class="grid grid-cols-2 gap-2">
          <button class="flex items-center gap-2 truncate rounded-lg bg-white/25 px-3 py-2 text-sm font-medium text-white">
            Living Room
          </button>
          <button class="flex items-center gap-2 truncate rounded-lg bg-black/20 px-3 py-2 text-sm font-medium text-white/70">
            Bedroom
          </button>
        </div>
      </div>
    </div>
  </div>

  <!-- Connector -->
  <div class="pointer-events-none flex justify-center">
    <div class="h-4 w-[4px] rounded-full bg-[var(--color-border)]"></div>
  </div>

  <!-- AND action card (Automation with method tabs) -->
  <div class="rounded-xl p-4 text-white" style="background-color: #8b5cf6">
    <div class="mb-3 flex items-center justify-between">
      <div class="flex items-center gap-2">
        <h3 class="text-2xl font-black text-white drop-shadow-sm">And</h3>
        <Terminal class="h-5 w-5 text-white/70" />
      </div>
      <div class="rounded p-1 text-white/40">
        <Trash2 class="h-4 w-4" />
      </div>
    </div>

    <!-- Method tabs -->
    <div class="mb-3 flex gap-1">
      <button class="rounded-lg bg-white/30 px-2 py-1 text-xs font-medium text-white">Shell</button>
      <button class="rounded-lg bg-black/20 px-2 py-1 text-xs font-medium text-white/60">AppleScript</button>
      <button class="rounded-lg bg-black/20 px-2 py-1 text-xs font-medium text-white/60">Shortcut</button>
      <button class="rounded-lg bg-black/20 px-2 py-1 text-xs font-medium text-white/60">Webhook</button>
    </div>

    <div class="flex items-start gap-2">
      <span class="shrink-0 pt-1.5 text-sm font-semibold drop-shadow-sm">Run</span>
      <div class="min-w-0 flex-1">
        <textarea
          readonly
          rows={2}
          class="w-full rounded-lg bg-black/20 px-2 py-1 font-mono text-sm text-white/90 outline-none placeholder:text-white/40 resize-none"
        >echo $TORRENT_NAME</textarea>
      </div>
    </div>
  </div>

  <!-- Connector -->
  <div class="pointer-events-none flex justify-center">
    <div class="h-4 w-[4px] rounded-full bg-[var(--color-border)]"></div>
  </div>

  <!-- Then add-action grid -->
  <div class="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4">
    <h3 class="mb-3 text-2xl font-black text-[var(--color-text-muted)]">Then</h3>
    <div class="grid grid-cols-2 gap-2">
      <button class="flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium text-white opacity-30 cursor-not-allowed" style="background-color: var(--color-primary)">
        <Cast class="h-4 w-4" />
        Cast
      </button>
      <button class="flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium text-white hover:opacity-80" style="background-color: var(--color-warning)">
        <FolderOutput class="h-4 w-4" />
        Move
      </button>
      <button class="flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium text-white hover:opacity-80" style="background-color: var(--color-success)">
        <Bell class="h-4 w-4" />
        Notify
      </button>
      <button class="flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium text-white hover:opacity-80" style="background-color: var(--color-error)">
        <MonitorPlay class="h-4 w-4" />
        Play
      </button>
      <button class="flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium text-white hover:opacity-80" style="background-color: #6366f1">
        <Subtitles class="h-4 w-4" />
        Subtitles
      </button>
      <button class="flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium text-white opacity-30 cursor-not-allowed" style="background-color: #8b5cf6">
        <Terminal class="h-4 w-4" />
        Automation
      </button>
      <button class="flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium text-white hover:opacity-80" style="background-color: #64748b">
        <Timer class="h-4 w-4" />
        Delay
      </button>
    </div>
  </div>
</div>
