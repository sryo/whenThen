<script lang="ts">
  import { FileVideo, File, Folder, ChevronDown, ChevronRight, Music, Clipboard, CheckSquare, Square, FolderOpen, FolderClosed } from "lucide-svelte";
  import type { TorrentFileInfo } from "$lib/types/torrent";
  import { formatBytes, useContextMenu } from "$lib/utils";
  import { torrentUpdateFiles } from "$lib/services/tauri-commands";
  import { uiState } from "$lib/state/ui.svelte";
  import ContextMenu from "$lib/components/common/ContextMenu.svelte";
  import type { ContextMenuEntry } from "$lib/types/ui";

  let {
    files,
    torrentId,
  }: {
    files: TorrentFileInfo[];
    torrentId: number;
  } = $props();

  let selectedIndices = $state<Set<number>>(new Set());
  let expandedFolders = $state<Set<string>>(new Set());

  $effect(() => {
    if (files.length > 0 && selectedIndices.size === 0) {
      selectedIndices = new Set(files.map((f) => f.index));
    }
  });

  interface FileNode {
    type: "file";
    file: TorrentFileInfo;
  }
  interface FolderNode {
    type: "folder";
    name: string;
    path: string;
    children: (FileNode | FolderNode)[];
    files: TorrentFileInfo[];
  }

  const fileTree = $derived(() => {
    const root: FolderNode = { type: "folder", name: "", path: "", children: [], files: [] };
    const folderMap = new Map<string, FolderNode>();
    folderMap.set("", root);

    for (const file of files) {
      const parts = file.path.split("/");
      if (parts.length === 1) {
        // Root-level file
        root.children.push({ type: "file", file });
        root.files.push(file);
      } else {
        // Build folder hierarchy
        let currentPath = "";
        let parent = root;
        for (let i = 0; i < parts.length - 1; i++) {
          const folderName = parts[i];
          currentPath = currentPath ? `${currentPath}/${folderName}` : folderName;
          let folder = folderMap.get(currentPath);
          if (!folder) {
            folder = { type: "folder", name: folderName, path: currentPath, children: [], files: [] };
            folderMap.set(currentPath, folder);
            parent.children.push(folder);
          }
          parent = folder;
        }
        parent.children.push({ type: "file", file });
        let cp = "";
        for (let i = 0; i < parts.length - 1; i++) {
          cp = cp ? `${cp}/${parts[i]}` : parts[i];
          const f = folderMap.get(cp);
          if (f) f.files.push(file);
        }
      }
    }

    // If there's only one top-level folder and no top-level files, skip it
    const topChildren = root.children.filter((c) => c.type === "folder") as FolderNode[];
    const topFiles = root.children.filter((c) => c.type === "file");
    if (topChildren.length === 1 && topFiles.length === 0) {
      return topChildren[0].children;
    }

    return root.children;
  });

  function isFileSelected(index: number): boolean {
    return selectedIndices.has(index);
  }

  function isFolderSelected(folderFiles: TorrentFileInfo[]): boolean {
    return folderFiles.every((f) => selectedIndices.has(f.index));
  }

  function isFolderPartial(folderFiles: TorrentFileInfo[]): boolean {
    const selected = folderFiles.filter((f) => selectedIndices.has(f.index));
    return selected.length > 0 && selected.length < folderFiles.length;
  }

  async function toggleFile(index: number) {
    const next = new Set(selectedIndices);
    if (next.has(index)) {
      next.delete(index);
    } else {
      next.add(index);
    }
    selectedIndices = next;
    await updateBackend();
  }

  async function toggleFolder(folderFiles: TorrentFileInfo[]) {
    const next = new Set(selectedIndices);
    const allSelected = folderFiles.every((f) => next.has(f.index));
    for (const f of folderFiles) {
      if (allSelected) {
        next.delete(f.index);
      } else {
        next.add(f.index);
      }
    }
    selectedIndices = next;
    await updateBackend();
  }

  function toggleFolderExpanded(path: string) {
    const next = new Set(expandedFolders);
    if (next.has(path)) {
      next.delete(path);
    } else {
      next.add(path);
    }
    expandedFolders = next;
  }

  async function updateBackend() {
    try {
      await torrentUpdateFiles(torrentId, Array.from(selectedIndices));
    } catch (err: any) {
      uiState.addToast(`Could not update files: ${err?.message || err}`, "error");
    }
  }

  async function selectOnly(index: number) {
    selectedIndices = new Set([index]);
    await updateBackend();
  }

  async function selectAllInFolder(folderFiles: TorrentFileInfo[]) {
    const next = new Set(selectedIndices);
    for (const f of folderFiles) next.add(f.index);
    selectedIndices = next;
    await updateBackend();
  }

  async function deselectAllInFolder(folderFiles: TorrentFileInfo[]) {
    const next = new Set(selectedIndices);
    for (const f of folderFiles) next.delete(f.index);
    selectedIndices = next;
    await updateBackend();
  }

  // Context menus
  const fileCtx = useContextMenu<TorrentFileInfo>();
  const folderCtx = useContextMenu<FolderNode>();

  function fileMenuItems(file: TorrentFileInfo): ContextMenuEntry[] {
    return [
      {
        icon: Clipboard,
        label: "Copy File Name",
        action: () => {
          navigator.clipboard.writeText(file.name);
          uiState.addToast("Copied to clipboard", "success");
        },
      },
      {
        icon: Clipboard,
        label: "Copy Path",
        action: () => {
          navigator.clipboard.writeText(file.path);
          uiState.addToast("Copied to clipboard", "success");
        },
      },
      { type: "divider" },
      {
        icon: CheckSquare,
        label: "Select Only This",
        action: () => selectOnly(file.index),
      },
      {
        icon: isFileSelected(file.index) ? Square : CheckSquare,
        label: isFileSelected(file.index) ? "Deselect" : "Select",
        action: () => toggleFile(file.index),
      },
    ];
  }

  function folderMenuItems(node: FolderNode): ContextMenuEntry[] {
    const isExpanded = expandedFolders.has(node.path);
    return [
      {
        icon: Clipboard,
        label: "Copy Folder Name",
        action: () => {
          navigator.clipboard.writeText(node.name);
          uiState.addToast("Copied to clipboard", "success");
        },
      },
      { type: "divider" },
      {
        icon: CheckSquare,
        label: "Select All in Folder",
        action: () => selectAllInFolder(node.files),
      },
      {
        icon: Square,
        label: "Deselect All in Folder",
        action: () => deselectAllInFolder(node.files),
      },
      { type: "divider" },
      {
        icon: isExpanded ? FolderClosed : FolderOpen,
        label: isExpanded ? "Collapse" : "Expand",
        action: () => toggleFolderExpanded(node.path),
      },
    ];
  }
</script>

{#snippet renderNode(node: FileNode | FolderNode, depth: number)}
  {#if node.type === "file"}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="flex items-center gap-2 rounded px-2 py-1 transition-colors hover:bg-[var(--color-bg-tertiary)]"
      style="padding-left: {depth * 16 + 8}px"
      oncontextmenu={(e) => fileCtx.open(e, node.file)}
    >
      <input
        type="checkbox"
        checked={isFileSelected(node.file.index)}
        onchange={() => toggleFile(node.file.index)}
        class="h-3.5 w-3.5 shrink-0 accent-[var(--color-primary)]"
      />
      {#if node.file.mime_type?.startsWith("audio/")}
        <Music class="h-3.5 w-3.5 shrink-0 text-[var(--color-primary)]" />
      {:else if node.file.is_playable}
        <FileVideo class="h-3.5 w-3.5 shrink-0 text-[var(--color-primary)]" />
      {:else}
        <File class="h-3.5 w-3.5 shrink-0 text-[var(--color-text-muted)]" />
      {/if}
      <span class="min-w-0 truncate text-xs text-[var(--color-text)] {!isFileSelected(node.file.index) ? 'opacity-50 line-through' : ''}">
        {node.file.name}
      </span>
      <span class="ml-auto shrink-0 text-[10px] text-[var(--color-text-muted)]">
        {formatBytes(node.file.length)}
      </span>
    </div>
  {:else}
    {@const isExpanded = expandedFolders.has(node.path)}
    <div>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="flex items-center gap-2 rounded px-2 py-1 transition-colors hover:bg-[var(--color-bg-tertiary)] cursor-pointer"
        style="padding-left: {depth * 16 + 8}px"
        oncontextmenu={(e) => folderCtx.open(e, node)}
      >
        <input
          type="checkbox"
          checked={isFolderSelected(node.files)}
          indeterminate={isFolderPartial(node.files)}
          onchange={() => toggleFolder(node.files)}
          class="h-3.5 w-3.5 shrink-0 accent-[var(--color-primary)]"
        />
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div class="flex min-w-0 flex-1 items-center gap-1.5" onclick={() => toggleFolderExpanded(node.path)}>
          {#if isExpanded}
            <ChevronDown class="h-3 w-3 shrink-0 text-[var(--color-text-muted)]" />
          {:else}
            <ChevronRight class="h-3 w-3 shrink-0 text-[var(--color-text-muted)]" />
          {/if}
          <Folder class="h-3.5 w-3.5 shrink-0 text-[var(--color-warning)]" />
          <span class="truncate text-xs font-medium text-[var(--color-text)]">
            {node.name}
          </span>
          <span class="ml-auto shrink-0 text-[10px] text-[var(--color-text-muted)]">
            {node.files.length} files
          </span>
        </div>
      </div>
      {#if isExpanded}
        {#each node.children as child}
          {@render renderNode(child, depth + 1)}
        {/each}
      {/if}
    </div>
  {/if}
{/snippet}

<div class="space-y-0.5">
  {#each fileTree() as node}
    {@render renderNode(node, 0)}
  {/each}
</div>

{#if fileCtx.state}
  <ContextMenu x={fileCtx.state.x} y={fileCtx.state.y} items={fileMenuItems(fileCtx.state.data)} onclose={fileCtx.close} />
{/if}

{#if folderCtx.state}
  <ContextMenu x={folderCtx.state.x} y={folderCtx.state.y} items={folderMenuItems(folderCtx.state.data)} onclose={folderCtx.close} />
{/if}
