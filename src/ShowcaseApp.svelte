<!-- Component showcase for visual verification. -->
<script lang="ts">
  import ActionBlock from "$lib/components/common/ActionBlock.svelte";
  import AnnotationLayer from "$lib/components/showcase/AnnotationLayer.svelte";
  import ShowcaseTitleBar from "$lib/components/showcase/ShowcaseTitleBar.svelte";
  import ShowcaseStatusBar from "$lib/components/showcase/ShowcaseStatusBar.svelte";
  import ShowcaseInputs from "$lib/components/showcase/ShowcaseInputs.svelte";
  import ShowcaseSchemeGrid from "$lib/components/showcase/ShowcaseSchemeGrid.svelte";
  import ShowcaseToasts from "$lib/components/showcase/ShowcaseToasts.svelte";
  import ShowcasePipelineCards from "$lib/components/showcase/ShowcasePipelineCards.svelte";
  import {
    Cast,
    FolderOutput,
    Bell,
    MonitorPlay,
    Subtitles,
    Terminal,
    Download,
  } from "lucide-svelte";

  type Status = "pending" | "running" | "done" | "failed" | "skipped";
  const statuses: Status[] = ["pending", "running", "done", "failed", "skipped"];
  const statusLabels: Record<Status, string> = {
    pending: "Pending",
    running: "Running",
    done: "Done",
    failed: "Failed",
    skipped: "Skipped",
  };

  const actions = [
    { icon: Download, color: "var(--color-info)", label: "Download", value: null },
    { icon: Cast, color: "var(--color-primary)", label: "Cast", value: "Living Room" },
    { icon: FolderOutput, color: "var(--color-warning)", label: "Move", value: "Movies" },
    { icon: Bell, color: "var(--color-success)", label: "Notify", value: null },
    { icon: MonitorPlay, color: "var(--color-error)", label: "Play", value: "VLC" },
    { icon: Subtitles, color: "#6366f1", label: "Subtitles", value: "en" },
    { icon: Terminal, color: "#8b5cf6", label: "Automation", value: "shell" },
  ];

  // Mock task scenarios for TaskCard-like pipeline strips
  const taskScenarios = [
    {
      name: "All done",
      steps: [
        { status: "done" as Status },
        { status: "done" as Status },
        { status: "done" as Status },
      ],
    },
    {
      name: "Running (mid-pipeline)",
      steps: [
        { status: "done" as Status },
        { status: "running" as Status },
        { status: "pending" as Status },
      ],
    },
    {
      name: "Failed at step 2",
      steps: [
        { status: "done" as Status },
        { status: "failed" as Status },
        { status: "pending" as Status },
      ],
    },
    {
      name: "Skipped at step 2",
      steps: [
        { status: "done" as Status },
        { status: "skipped" as Status },
        { status: "done" as Status },
      ],
    },
    {
      name: "Mixed skip + fail",
      steps: [
        { status: "done" as Status },
        { status: "skipped" as Status },
        { status: "failed" as Status },
      ],
    },
    {
      name: "All skipped",
      steps: [
        { status: "done" as Status },
        { status: "skipped" as Status },
        { status: "skipped" as Status },
      ],
    },
  ];

  // Force dark scheme for showcase
  let isDark = $state(true);

  function toggleTheme() {
    isDark = !isDark;
    document.documentElement.style.colorScheme = isDark ? "dark" : "light";
  }
</script>

<div
  class="relative h-screen overflow-y-auto p-8"
  style="background-color: var(--color-bg); color: var(--color-text);"
  style:color-scheme={isDark ? "dark" : "light"}
>
  <div class="mx-auto max-w-4xl space-y-12">
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-bold">Component Showcase</h1>
      <button
        onclick={toggleTheme}
        class="rounded-lg border border-[var(--color-border)] px-3 py-1.5 text-sm text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-tertiary)]"
      >
        {isDark ? "Light" : "Dark"}
      </button>
    </div>

    <!-- ── Foundation ───────────────────────────── -->
    <div class="border-t border-[var(--color-border)] pt-8">
      <h2 class="text-xs font-medium uppercase tracking-wide text-[var(--color-text-muted)]">Foundation</h2>
      <p class="mt-1 text-xs text-[var(--color-text-muted)]">Colors and themes</p>
    </div>

    <section>
      <h3 class="mb-4 text-lg font-semibold">Status Colors</h3>
      <div class="flex flex-wrap gap-3 rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4">
        <div class="flex items-center gap-2">
          <div class="h-4 w-4 rounded-full bg-[var(--color-success)]"></div>
          <span class="text-sm text-[var(--color-text-secondary)]">Done (green)</span>
        </div>
        <div class="flex items-center gap-2">
          <div class="h-4 w-4 rounded-full bg-[var(--color-primary)]"></div>
          <span class="text-sm text-[var(--color-text-secondary)]">Running (primary)</span>
        </div>
        <div class="flex items-center gap-2">
          <div class="h-4 w-4 rounded-full bg-[var(--color-bg-tertiary)]"></div>
          <span class="text-sm text-[var(--color-text-secondary)]">Pending (muted)</span>
        </div>
        <div class="flex items-center gap-2">
          <div class="h-4 w-4 rounded-full bg-[var(--color-error)]"></div>
          <span class="text-sm text-[var(--color-text-secondary)]">Failed (red)</span>
        </div>
        <div class="flex items-center gap-2">
          <div class="h-4 w-4 rounded-full bg-[var(--color-warning)]"></div>
          <span class="text-sm text-[var(--color-text-secondary)]">Skipped (amber)</span>
        </div>
      </div>
    </section>

    <!-- ── Elements ─────────────────────────────── -->
    <div class="border-t border-[var(--color-border)] pt-8">
      <h2 class="text-xs font-medium uppercase tracking-wide text-[var(--color-text-muted)]">Elements</h2>
      <p class="mt-1 text-xs text-[var(--color-text-muted)]">Standalone UI controls</p>
    </div>

    <section>
      <h3 class="mb-4 text-lg font-semibold">Color Scheme Grid</h3>
      <div class="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4">
        <ShowcaseSchemeGrid />
      </div>
    </section>

    <section>
      <h3 class="mb-4 text-lg font-semibold">Inputs &amp; Controls</h3>
      <div class="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4">
        <ShowcaseInputs />
      </div>
    </section>

    <section>
      <h3 class="mb-4 text-lg font-semibold">Toast Variants</h3>
      <div class="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4">
        <ShowcaseToasts />
      </div>
    </section>

    <section>
      <h3 class="mb-4 text-lg font-semibold">ActionBlock &mdash; pip</h3>
      <div class="space-y-3 rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4">
        {#each statuses as status}
          <div>
            <span class="mb-1 block text-xs font-medium text-[var(--color-text-muted)]">{statusLabels[status]}</span>
            <div class="flex flex-wrap items-center gap-3">
              {#each actions as action}
                <ActionBlock icon={action.icon} color={action.color} label={action.label} value={action.value} size="pip" {status} />
              {/each}
            </div>
          </div>
        {/each}
      </div>
    </section>

    <section>
      <h3 class="mb-4 text-lg font-semibold">ActionBlock &mdash; sm</h3>
      <div class="space-y-3 rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4">
        {#each statuses as status}
          <div>
            <span class="mb-1 block text-xs font-medium text-[var(--color-text-muted)]">{statusLabels[status]}</span>
            <div class="flex flex-wrap items-center gap-2">
              {#each actions as action}
                <ActionBlock icon={action.icon} color={action.color} label={action.label} value={action.value} size="sm" {status} />
              {/each}
            </div>
          </div>
        {/each}
      </div>
    </section>

    <section>
      <h3 class="mb-4 text-lg font-semibold">ActionBlock &mdash; md</h3>
      <div class="space-y-3 rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4">
        {#each statuses as status}
          <div>
            <span class="mb-1 block text-xs font-medium text-[var(--color-text-muted)]">{statusLabels[status]}</span>
            <div class="flex flex-wrap items-center gap-2">
              {#each actions as action}
                <ActionBlock icon={action.icon} color={action.color} label={action.label} value={action.value} size="md" {status} />
              {/each}
            </div>
          </div>
        {/each}
      </div>
    </section>

    <!-- ── Compositions ─────────────────────────── -->
    <div class="border-t border-[var(--color-border)] pt-8">
      <h2 class="text-xs font-medium uppercase tracking-wide text-[var(--color-text-muted)]">Compositions</h2>
      <p class="mt-1 text-xs text-[var(--color-text-muted)]">Combined components and app regions</p>
    </div>

    <section>
      <h3 class="mb-4 text-lg font-semibold">Pipeline Strips</h3>
      <div class="space-y-3 rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4">
        {#each taskScenarios as scenario}
          <div class="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] p-3">
            <div class="mb-2 text-xs font-medium text-[var(--color-text-muted)]">{scenario.name}</div>
            <div class="flex items-center">
              {#each scenario.steps as step, i}
                {#if i > 0}
                  <div class="h-[2px] w-2 rounded-full bg-[var(--color-border)]"></div>
                {/if}
                <ActionBlock
                  icon={actions[i].icon}
                  color={actions[i].color}
                  label={actions[i].label}
                  value={actions[i].value}
                  size="pip"
                  status={step.status}
                />
              {/each}
            </div>
          </div>
        {/each}
      </div>
    </section>

    <section>
      <h3 class="mb-4 text-lg font-semibold">Pipeline Cards</h3>
      <div class="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4">
        <ShowcasePipelineCards />
      </div>
    </section>

    <section>
      <h3 class="mb-4 text-lg font-semibold">TitleBar</h3>
      <div class="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4">
        <ShowcaseTitleBar />
      </div>
    </section>

    <section>
      <h3 class="mb-4 text-lg font-semibold">StatusBar</h3>
      <div class="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4">
        <ShowcaseStatusBar />
      </div>
    </section>

    <div class="h-8"></div>
  </div>

  <AnnotationLayer />
</div>
