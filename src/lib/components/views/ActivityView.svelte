<script lang="ts">
  import { Trash2 } from "lucide-svelte";
  import TaskCard from "$lib/components/common/TaskCard.svelte";
  import { tasksState } from "$lib/state/tasks.svelte";
</script>

<div class="mx-auto max-w-2xl space-y-4 p-6">
  <!-- Active Tasks -->
  <div>
    {#if tasksState.activeTasks.length > 0}
      <div class="space-y-2">
        {#each tasksState.activeTasks as task (task.id)}
          <TaskCard {task} />
        {/each}
      </div>
    {:else}
      <p class="py-8 text-center text-sm text-[var(--color-text-muted)]">
        Nothing running. Drop a torrent on a playlet to start.
      </p>
    {/if}
  </div>

  <!-- Completed Tasks -->
  {#if tasksState.completedTasks.length > 0}
    <div>
      <div class="mb-3 flex items-center justify-between">
        <h2 class="text-sm font-semibold uppercase tracking-wider text-[var(--color-text-muted)]">
          Done
        </h2>
        <button
          onclick={() => tasksState.clearCompleted()}
          class="flex items-center gap-1 rounded-md px-2 py-1 text-xs text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
        >
          <Trash2 class="h-3 w-3" />
          Clear
        </button>
      </div>
      <div class="space-y-2">
        {#each tasksState.completedTasks as task (task.id)}
          <TaskCard {task} />
        {/each}
      </div>
    </div>
  {/if}
</div>
