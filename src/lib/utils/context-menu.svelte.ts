// Reactive context menu helper for Svelte 5 components.

export interface ContextMenuState<T = undefined> {
  x: number;
  y: number;
  data: T;
}

export function useContextMenu<T = undefined>() {
  let state = $state<ContextMenuState<T> | null>(null);

  function open(e: MouseEvent, data?: T) {
    e.preventDefault();
    e.stopPropagation();
    state = { x: e.clientX, y: e.clientY, data: data as T };
  }

  function close() {
    state = null;
  }

  return {
    get state() {
      return state;
    },
    open,
    close,
  };
}
