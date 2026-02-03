import type { SubtitleInfo } from "$lib/types/playback";

let currentSubtitle = $state<SubtitleInfo | null>(null);

export const subtitlesState = {
  get current() {
    return currentSubtitle;
  },
  get hasSubtitle() {
    return currentSubtitle !== null;
  },
  get url() {
    return currentSubtitle?.url ?? null;
  },

  setSubtitle(info: SubtitleInfo) {
    currentSubtitle = info;
  },

  clear() {
    currentSubtitle = null;
  },
};
