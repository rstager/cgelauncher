export type DiskStatus = 'READY' | 'CREATING' | 'RESTORING' | 'FAILED' | 'DELETING';

export type DiskType = 'pd-standard' | 'pd-ssd' | 'pd-balanced';

export interface Disk {
  name: string;
  sizeGb: number;
  status: DiskStatus;
  type: DiskType;
  attachedTo: string | null;
}

export type VmStatus = 'Running' | 'Starting' | 'Stopping' | 'Stopped' | 'NotFound';

export interface MachineConfig {
  machineType: string;
  gpuType: string | null;
  gpuCount: number | null;
  spot: boolean;
}

export interface PricingLineItem {
  label: string;
  spotCost: number;
  ondemandCost: number;
}

export interface PricingEstimate {
  spotHourly: number;
  ondemandHourly: number;
  currency: string;
  breakdown: PricingLineItem[];
}

export interface ConfigPreset {
  name: string;
  machineType: string;
  gpuType: string | null;
  gpuCount: number | null;
  description: string;
}

export interface DiskConfig {
  machineType: string;
  gpuType: string | null;
  gpuCount: number | null;
  spot: boolean;
}

export interface UserPreferences {
  project: string;
  zone: string;
  defaultMachineType: string | null;
  defaultGpuType: string | null;
  defaultGpuCount: number | null;
  defaultSpot: boolean;
  executionMode: 'gcloud' | 'api';
  apiAccessToken: string | null;
  customPresets: ConfigPreset[];
  hiddenPresets: string[];
  diskConfigs: Record<string, DiskConfig>;
}

export interface AuthStatus {
  authenticated: boolean;
  method: 'gcloud' | 'oauth2' | string;
  account: string | null;
}

export interface VmStatusUpdate {
  diskName: string;
  instanceName: string;
  status: VmStatus;
  machineType: string | null;
  gpuType: string | null;
  gpuCount: number | null;
  memoryGb: number | null;
}

export interface GcloudError {
  message: string;
  command: string;
  exitCode: number;
}

export interface GcloudCommandLogEntry {
  command: string;
  response: string;
  success: boolean;
  exitCode: number;
}
