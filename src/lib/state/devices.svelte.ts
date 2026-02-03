import type {
  ChromecastDeviceInfo,
  DeviceStatus,
} from "$lib/types/device";

let devices = $state<ChromecastDeviceInfo[]>([]);
let isDiscovering = $state(false);

export const devicesState = {
  get devices() {
    return devices;
  },
  get isDiscovering() {
    return isDiscovering;
  },
  get connectedDevices() {
    return devices.filter((d) => d.status === "connected");
  },
  get hasConnectedDevice() {
    return devices.some((d) => d.status === "connected");
  },

  setDiscovering(value: boolean) {
    isDiscovering = value;
  },

  addDevice(device: ChromecastDeviceInfo) {
    const existing = devices.findIndex((d) => d.id === device.id);
    if (existing >= 0) {
      devices[existing] = device;
    } else {
      devices = [...devices, device];
    }
  },

  removeDevice(id: string) {
    devices = devices.filter((d) => d.id !== id);
  },

  updateDeviceStatus(id: string, status: DeviceStatus) {
    const idx = devices.findIndex((d) => d.id === id);
    if (idx >= 0) {
      devices[idx] = { ...devices[idx], status };
    }
  },

  setDevices(newDevices: ChromecastDeviceInfo[]) {
    devices = newDevices;
  },

  clear() {
    devices = [];
    isDiscovering = false;
  },
};
