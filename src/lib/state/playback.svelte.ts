import type {
  PlaybackState,
  PlaybackStatusResponse,
} from "$lib/types/playback";

let status = $state<PlaybackStatusResponse | null>(null);
let isLoading = $state(false);
let activeTorrentName = $state<string | null>(null);
let activeDeviceName = $state<string | null>(null);

export const playbackState = {
  get status() {
    return status;
  },
  get isLoading() {
    return isLoading;
  },
  get isPlaying() {
    return status?.state === "playing";
  },
  get isPaused() {
    return status?.state === "paused";
  },
  get isIdle() {
    return !status || status.state === "idle";
  },
  get currentTime() {
    return status?.current_time ?? 0;
  },
  get duration() {
    return status?.duration ?? 0;
  },
  get volume() {
    return status?.volume ?? 1;
  },
  get activeDeviceId() {
    return status?.device_id ?? null;
  },
  get activeTorrentName() {
    return activeTorrentName;
  },
  get activeDeviceName() {
    return activeDeviceName;
  },

  setStatus(newStatus: PlaybackStatusResponse) {
    status = newStatus;
  },

  setLoading(value: boolean) {
    isLoading = value;
  },

  setContext(torrentName: string | null, deviceName: string | null) {
    activeTorrentName = torrentName;
    activeDeviceName = deviceName;
  },

  clear() {
    status = null;
    isLoading = false;
    activeTorrentName = null;
    activeDeviceName = null;
  },
};
