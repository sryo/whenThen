export interface ChromecastDeviceInfo {
  id: string;
  name: string;
  model: string;
  address: string;
  port: number;
  status: DeviceStatus;
}

export type DeviceStatus = "discovered" | "connecting" | "connected" | "error";

export interface DeviceFoundEvent {
  id: string;
  name: string;
  model: string;
  address: string;
  port: number;
}

export interface DeviceLostEvent {
  id: string;
}

export interface DeviceConnectedEvent {
  id: string;
  name: string;
}

export interface DeviceDisconnectedEvent {
  id: string;
  reason: string;
}

