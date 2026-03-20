import { invoke } from '@tauri-apps/api/core';
import type {
  Disk,
  DiskConfig,
  ImageInfo,
  MachineConfig,
  ConfigPreset,
  PricingEstimate,
  AuthStatus,
  UserPreferences,
  VmStatusUpdate,
  GcloudCommandLogEntry,
} from './types.ts';

function assertTauriRuntime(): void {
  const tauriInternals = (globalThis as { __TAURI_INTERNALS__?: { invoke?: unknown } })
    .__TAURI_INTERNALS__;

  if (!tauriInternals || typeof tauriInternals.invoke !== 'function') {
    throw new Error(
      'Tauri runtime is not available. Start the desktop app with `cargo tauri dev` instead of `npm run dev`.',
    );
  }
}

async function tauriInvoke<T>(command: string, payload?: Record<string, unknown>): Promise<T> {
  assertTauriRuntime();
  return invoke<T>(command, payload);
}

export async function listDisks(): Promise<Disk[]> {
  return tauriInvoke<Disk[]>('list_disks');
}

export async function createDisk(diskName: string, sizeGb: number, diskType: string, sourceImage?: string): Promise<void> {
  return tauriInvoke<void>('create_disk', { diskName, sizeGb, diskType, sourceImage: sourceImage ?? null });
}

export async function listImages(imageProject: string, filter?: string): Promise<ImageInfo[]> {
  return tauriInvoke<ImageInfo[]>('list_images', { imageProject, filter: filter ?? null });
}

export async function deleteDisk(diskName: string): Promise<void> {
  return tauriInvoke<void>('delete_disk', { diskName });
}

export async function startVm(diskName: string, config: MachineConfig): Promise<VmStatusUpdate> {
  return tauriInvoke<VmStatusUpdate>('start_vm', { diskName, config });
}

export async function stopVm(instanceName: string): Promise<{ success: boolean }> {
  return tauriInvoke<{ success: boolean }>('stop_vm', { instanceName });
}

export async function estimatePricing(config: MachineConfig): Promise<PricingEstimate> {
  return tauriInvoke<PricingEstimate>('estimate_pricing', { config });
}

export async function checkAuth(): Promise<AuthStatus> {
  return tauriInvoke<AuthStatus>('check_auth');
}

export async function configureSsh(instanceName: string): Promise<{ sshHost: string; configPath: string }> {
  return tauriInvoke<{ sshHost: string; configPath: string }>('configure_ssh', { instanceName });
}

export async function launchSshTerminal(instanceName: string): Promise<void> {
  return tauriInvoke<void>('launch_ssh_terminal', { instanceName });
}

export async function getPreferences(): Promise<UserPreferences> {
  return tauriInvoke<UserPreferences>('get_preferences');
}

export async function setPreferences(preferences: UserPreferences): Promise<UserPreferences> {
  return tauriInvoke<UserPreferences>('set_preferences', { preferences });
}

export async function saveDiskConfig(diskName: string, config: DiskConfig): Promise<void> {
  return tauriInvoke<void>('save_disk_config', { diskName, config });
}

export async function getDiskConfig(diskName: string): Promise<DiskConfig | null> {
  return tauriInvoke<DiskConfig | null>('get_disk_config', { diskName });
}

export async function saveCustomPreset(preset: ConfigPreset): Promise<ConfigPreset[]> {
  return tauriInvoke<ConfigPreset[]>('save_custom_preset', { preset });
}

export async function deleteCustomPreset(name: string): Promise<ConfigPreset[]> {
  return tauriInvoke<ConfigPreset[]>('delete_custom_preset', { name });
}

export async function getGcloudLogs(): Promise<GcloudCommandLogEntry[]> {
  return tauriInvoke<GcloudCommandLogEntry[]>('get_gcloud_logs');
}

export async function startOAuthLogin(): Promise<AuthStatus> {
  return tauriInvoke<AuthStatus>('start_oauth_login');
}

export async function revokeOauth(): Promise<void> {
  return tauriInvoke<void>('revoke_oauth');
}
