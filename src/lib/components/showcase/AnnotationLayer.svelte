<!-- Overlay for dropping numbered pins with comments on showcase elements. -->
<script lang="ts">
  import { onMount, tick } from "svelte";

  interface Pin {
    id: number;
    x: number;
    y: number;
    note: string;
    section?: string;
  }

  const STORAGE_KEY = "showcase-pins";
  const API_URL = "/api/showcase-feedback";
  const DRAG_THRESHOLD = 3;

  let pins = $state<Pin[]>([]);
  let editingId = $state<number | null>(null);
  let editText = $state("");
  let overlay: HTMLDivElement | undefined = $state();
  let copyLabel = $state("Copy notes");
  let annotating = $state(false);
  let overlayHeight = $state(0);
  let drag = $state<{
    pinId: number;
    startX: number;
    startY: number;
    moved: boolean;
    clientX: number;
    clientY: number;
  } | null>(null);
  let hoveredPinId = $state<number | null>(null);

  function getScrollContainer(): HTMLElement | null {
    return overlay?.closest(".overflow-y-auto") as HTMLElement | null;
  }

  function syncHeight() {
    const container = getScrollContainer();
    if (container) overlayHeight = container.scrollHeight;
  }

  onMount(() => {
    loadPins();
    syncHeight();

    const container = getScrollContainer();
    const ro = new ResizeObserver(syncHeight);
    if (container) ro.observe(container);

    function onPointerMove(e: PointerEvent) {
      if (!drag || !overlay) return;
      const dx = e.clientX - drag.startX;
      const dy = e.clientY - drag.startY;
      if (!drag.moved && Math.abs(dx) < DRAG_THRESHOLD && Math.abs(dy) < DRAG_THRESHOLD) return;
      drag.moved = true;
      drag.clientX = e.clientX;
      drag.clientY = e.clientY;

      const rect = overlay.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      pins = pins.map((p) => (p.id === drag!.pinId ? { ...p, x, y } : p));
    }

    function onPointerUp() {
      if (!drag) return;
      if (drag.moved) {
        const section = detectContext(drag.clientX, drag.clientY);
        pins = pins.map((p) => (p.id === drag!.pinId ? { ...p, section } : p));
        savePinsLocal();
      }
      drag = null;
    }

    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", onPointerUp);
    return () => {
      ro.disconnect();
      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("pointerup", onPointerUp);
    };
  });

  async function loadPins() {
    try {
      const res = await fetch(API_URL);
      if (res.ok) {
        const data = await res.json();
        if (Array.isArray(data) && data.length > 0) {
          pins = data;
          return;
        }
      }
    } catch {
      // API not available
    }
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (raw) pins = JSON.parse(raw);
    } catch {
      // ignore
    }
  }

  function savePinsLocal() {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(pins));
  }

  // Fallback: heading-only detection from scroll position
  function detectSectionHeading(y: number): string {
    const container = getScrollContainer();
    if (!container) return "";
    const sections = container.querySelectorAll("section");
    let best = "";
    for (const sec of sections) {
      const heading = sec.querySelector("h3") ?? sec.querySelector("h2");
      if (!heading) continue;
      if (sec.offsetTop <= y) best = heading.textContent?.trim() ?? "";
    }
    return best;
  }

  // Rich detection: section / subsection label / element text
  function detectContext(clientX: number, clientY: number): string {
    if (!overlay) return "";

    // Hide overlay so elementFromPoint hits the actual showcase content
    overlay.style.display = "none";
    const el = document.elementFromPoint(clientX, clientY);
    overlay.style.display = "";

    if (!el) return "";

    const parts: string[] = [];

    // Section heading (h2/h3 inside nearest <section>)
    const section = el.closest("section");
    if (section) {
      const heading = section.querySelector("h3") ?? section.querySelector("h2");
      if (heading) parts.push(heading.textContent?.trim() ?? "");
    }

    // Walk up to find a subsection label (span/label with text-sm class)
    let node: Element | null = el;
    let found = false;
    while (node && node !== section && !found) {
      const parent = node.parentElement;
      if (parent) {
        for (const child of parent.children) {
          if (child === node) continue;
          if (
            (child.tagName === "LABEL" || child.tagName === "SPAN") &&
            child.className.includes("text-sm") &&
            child.textContent?.trim()
          ) {
            const text = child.textContent.trim();
            if (!parts.includes(text)) parts.push(text);
            found = true;
            break;
          }
        }
      }
      node = node.parentElement;
    }

    // Element's own visible label
    const label = elementLabel(el);
    if (label && !parts.includes(label)) parts.push(label);

    return parts.filter(Boolean).join(" / ");
  }

  function elementLabel(el: Element): string {
    const btn = el.tagName === "BUTTON" ? el : el.closest("button");
    if (btn) {
      const text = btn.textContent?.trim() ?? "";
      return text.length > 40 ? text.slice(0, 40) + "\u2026" : text;
    }
    if (el.tagName === "INPUT") {
      const input = el as HTMLInputElement;
      if (input.placeholder) return input.placeholder;
      if (input.id) {
        const lbl = document.querySelector(`label[for="${input.id}"]`);
        if (lbl) return lbl.textContent?.trim() ?? "";
      }
      return input.type || "input";
    }
    if (el.tagName === "SELECT") return "select";
    if (el.tagName === "TEXTAREA") return "textarea";
    const text = el.textContent?.trim() ?? "";
    if (text.length > 0 && text.length <= 40) return text;
    return "";
  }

  function handleClick(e: MouseEvent) {
    if ((e.target as HTMLElement).closest("[data-annotation-toolbar]")) return;
    if (!overlay) return;

    const rect = overlay.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const nextId = pins.length > 0 ? Math.max(...pins.map((p) => p.id)) + 1 : 1;
    const section = detectContext(e.clientX, e.clientY);
    const pin: Pin = { id: nextId, x, y, note: "", section };
    pins = [...pins, pin];
    editingId = nextId;
    editText = "";
    savePinsLocal();
  }

  function startDrag(e: PointerEvent, pinId: number) {
    e.stopPropagation();
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    drag = {
      pinId,
      startX: e.clientX,
      startY: e.clientY,
      moved: false,
      clientX: e.clientX,
      clientY: e.clientY,
    };
  }

  function handlePinClick(e: MouseEvent, pin: Pin) {
    e.stopPropagation();
    if (drag?.moved) return;
    editingId = editingId === pin.id ? null : pin.id;
    editText = pin.note;
  }

  function toggleAnnotating() {
    annotating = !annotating;
    if (!annotating) editingId = null;
  }

  function commitNote(pinId: number) {
    pins = pins.map((p) => (p.id === pinId ? { ...p, note: editText } : p));
    editingId = null;
    savePinsLocal();
  }

  function removePin(pinId: number) {
    pins = pins.filter((p) => p.id !== pinId);
    if (editingId === pinId) editingId = null;
    savePinsLocal();
  }

  function clearAll() {
    pins = [];
    editingId = null;
    savePinsLocal();
  }

  function handleKeydown(e: KeyboardEvent, pinId: number) {
    if (e.key === "Enter") {
      e.preventDefault();
      commitNote(pinId);
    } else if (e.key === "Escape") {
      removePin(pinId);
    }
  }

  function handleBlur(pinId: number) {
    if (!editText.trim()) {
      removePin(pinId);
    } else {
      commitNote(pinId);
    }
  }

  function buildNotesText(): string {
    if (pins.length === 0) return "";
    const lines = pins.map((p) => {
      const section = p.section ? ` [${p.section}]` : "";
      const note = p.note || "(no note)";
      return `${p.id}.${section} ${note}`;
    });
    return "Showcase feedback:\n" + lines.join("\n");
  }

  async function copyNotes() {
    const text = buildNotesText();
    if (!text) return;
    try {
      await navigator.clipboard.writeText(text);
      copyLabel = "Copied!";
      setTimeout(() => (copyLabel = "Copy notes"), 1500);
    } catch {
      // Clipboard API may not be available
    }
  }

  async function saveToFile() {
    const enriched = pins.map((p) => ({
      ...p,
      section: p.section || detectSectionHeading(p.y),
    }));
    try {
      await fetch(API_URL, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(enriched),
      });
      pins = enriched;
      savePinsLocal();
    } catch {
      // API not available
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  bind:this={overlay}
  class="absolute top-0 left-0 right-0 z-[9998]"
  class:cursor-crosshair={annotating && !drag}
  class:cursor-grabbing={!!drag}
  style="height: {overlayHeight}px; pointer-events: {annotating ? 'auto' : 'none'};"
  onclick={handleClick}
>
  {#each pins as pin (pin.id)}
    <div
      class="absolute -translate-x-1/2 -translate-y-1/2 transition-opacity"
      class:pointer-events-auto={annotating}
      class:pointer-events-none={!annotating}
      class:opacity-40={!annotating}
      style="left: {pin.x}px; top: {pin.y}px;"
      onpointerenter={() => { if (!drag) hoveredPinId = pin.id; }}
      onpointerleave={() => { if (hoveredPinId === pin.id) hoveredPinId = null; }}
    >
      <!-- Pin circle -->
      <button
        class="relative flex h-6 w-6 items-center justify-center rounded-full bg-blue-500 text-xs font-bold text-white shadow-lg transition-colors hover:bg-blue-600"
        class:cursor-grab={annotating && !drag}
        class:cursor-grabbing={drag?.pinId === pin.id}
        onpointerdown={(e) => startDrag(e, pin.id)}
        onclick={(e) => handlePinClick(e, pin)}
        title={pin.section ? `[${pin.section}] ${pin.note || "No note"}` : (pin.note || "No note")}
      >
        {pin.id}
      </button>

      <!-- Delete badge on hover -->
      {#if hoveredPinId === pin.id && !drag && editingId !== pin.id}
        <button
          class="absolute -top-1.5 -right-1.5 flex h-3.5 w-3.5 items-center justify-center rounded-full bg-red-500 text-[8px] leading-none font-bold text-white shadow hover:bg-red-600"
          onclick={(e) => { e.stopPropagation(); removePin(pin.id); }}
          title="Delete pin"
        >
          ×
        </button>
      {/if}

      {#if annotating && editingId === pin.id}
        <!-- svelte-ignore a11y_autofocus -->
        <div class="absolute left-8 top-0 z-10 w-48 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] p-2 shadow-xl" onclick={(e) => e.stopPropagation()}>
          <input
            type="text"
            autofocus
            bind:value={editText}
            onkeydown={(e) => handleKeydown(e, pin.id)}
            onblur={() => handleBlur(pin.id)}
            class="w-full rounded border border-[var(--color-border)] bg-[var(--color-bg-secondary)] px-2 py-1 text-xs text-[var(--color-text)] focus:outline-none"
            placeholder="Add a note..."
          />
        </div>
      {:else if annotating && pin.note}
        <div class="pointer-events-none absolute left-8 top-0 max-w-48 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-2 py-1 text-xs text-[var(--color-text)] shadow-lg">
          {pin.note}
        </div>
      {/if}
    </div>
  {/each}
</div>

<!-- Toolbar -->
<div class="fixed bottom-4 right-4 z-[9999] flex items-center gap-2" data-annotation-toolbar>
  {#if pins.length > 0}
    <button
      onclick={saveToFile}
      class="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-1.5 text-xs text-[var(--color-text-secondary)] shadow-lg hover:bg-[var(--color-bg-tertiary)]"
    >
      Save
    </button>
    <button
      onclick={copyNotes}
      class="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-1.5 text-xs text-[var(--color-text-secondary)] shadow-lg hover:bg-[var(--color-bg-tertiary)]"
    >
      {copyLabel}
    </button>
    <button
      onclick={clearAll}
      class="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-1.5 text-xs text-[var(--color-text-secondary)] shadow-lg hover:bg-[var(--color-bg-tertiary)]"
    >
      Clear pins
    </button>
  {/if}
  <button
    onclick={toggleAnnotating}
    class="rounded-lg border px-3 py-1.5 text-xs font-medium shadow-lg transition-colors {annotating
      ? 'border-blue-500/50 bg-blue-500/15 text-blue-400'
      : 'border-[var(--color-border)] bg-[var(--color-bg)] text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-tertiary)]'}"
  >
    {annotating ? "Annotating" : "Annotate"}
  </button>
</div>
