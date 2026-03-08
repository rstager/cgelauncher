import { useState, useCallback, useMemo } from 'react';
import type { MachineConfig, ConfigPreset, UserPreferences } from './lib/types.ts';
import { startVm, stopVm, saveDiskConfig, saveCustomPreset, deleteCustomPreset } from './lib/tauri.ts';
import { useDisks } from './hooks/useDisks.ts';
import { useVmStatus } from './hooks/useVmStatus.ts';
import { usePricing } from './hooks/usePricing.ts';
import { useConfig } from './hooks/useConfig.ts';
import Layout from './components/Layout.tsx';

const DEFAULT_CONFIG: MachineConfig = {
  machineType: 'n1-standard-8',
  gpuType: 'nvidia-tesla-t4',
  gpuCount: 4,
  spot: true,
};

export default function App() {
  const { disks, loading: disksLoading, refresh: refreshDisks } = useDisks();
  const { statuses: vmStatuses, upsertStatus } = useVmStatus();
  const { preferences, save: savePreferences } = useConfig();
  const [selectedDisk, setSelectedDisk] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  // Build initial config from preferences when they load
  const initialConfig = useMemo((): MachineConfig => ({
    machineType: preferences.defaultMachineType ?? DEFAULT_CONFIG.machineType,
    gpuType: preferences.defaultGpuType ?? DEFAULT_CONFIG.gpuType,
    gpuCount: preferences.defaultGpuCount ?? DEFAULT_CONFIG.gpuCount,
    spot: preferences.defaultSpot ?? DEFAULT_CONFIG.spot,
  }), [preferences]);

  const [config, setConfig] = useState<MachineConfig>(DEFAULT_CONFIG);
  const [configInitialized, setConfigInitialized] = useState(false);

  // Sync config with preferences once on load
  if (!configInitialized && preferences.project !== '') {
    setConfig(initialConfig);
    setConfigInitialized(true);
  }

  const { pricing, loading: pricingLoading } = usePricing(config);

  const handleSelectDisk = useCallback((name: string) => {
    setSelectedDisk(name);
    // Restore last-used config for this disk if available
    const diskConfig = preferences.diskConfigs?.[name];
    if (diskConfig) {
      setConfig({
        machineType: diskConfig.machineType,
        gpuType: diskConfig.gpuType,
        gpuCount: diskConfig.gpuCount,
        spot: diskConfig.spot,
      });
    }
  }, [preferences.diskConfigs]);

  const handleStartVm = useCallback(async () => {
    if (!selectedDisk) return;
    setActionError(null);
    upsertStatus({
      diskName: selectedDisk,
      instanceName: `${selectedDisk}-vm`,
      status: 'Starting',
      machineType: config.machineType,
      gpuType: config.gpuType,
      gpuCount: config.gpuCount,
      memoryGb: null,
    });
    try {
      // Save this config as the disk's last-used config
      void saveDiskConfig(selectedDisk, {
        machineType: config.machineType,
        gpuType: config.gpuType,
        gpuCount: config.gpuCount,
        spot: config.spot,
      });
      const update = await startVm(selectedDisk, config);
      upsertStatus(update);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setActionError(`Failed to start VM: ${message}`);
      upsertStatus({
        diskName: selectedDisk,
        instanceName: `${selectedDisk}-vm`,
        status: 'Stopped',
        machineType: null,
        gpuType: null,
        gpuCount: null,
        memoryGb: null,
      });
      console.error('Failed to start VM:', err);
    }
  }, [selectedDisk, config, upsertStatus]);

  const handleStopVm = useCallback(async () => {
    if (!selectedDisk) return;
    setActionError(null);
    const vmName = selectedDisk;
    upsertStatus({
      diskName: selectedDisk,
      instanceName: vmName,
      status: 'Stopping',
      machineType: config.machineType,
      gpuType: config.gpuType,
      gpuCount: config.gpuCount,
      memoryGb: null,
    });
    try {
      await stopVm(vmName);
      upsertStatus({
        diskName: selectedDisk,
        instanceName: vmName,
        status: 'Stopped',
        machineType: null,
        gpuType: null,
        gpuCount: null,
        memoryGb: null,
      });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setActionError(`Failed to stop VM: ${message}`);
      upsertStatus({
        diskName: selectedDisk,
        instanceName: vmName,
        status: 'Running',
        machineType: config.machineType,
        gpuType: config.gpuType,
        gpuCount: config.gpuCount,
        memoryGb: null,
      });
      console.error('Failed to stop VM:', err);
    }
  }, [selectedDisk, config.machineType, config.gpuType, config.gpuCount, upsertStatus]);

  const [customPresets, setCustomPresets] = useState<ConfigPreset[]>(
    preferences.customPresets ?? [],
  );
  const [hiddenPresets, setHiddenPresets] = useState<string[]>(
    preferences.hiddenPresets ?? [],
  );

  // Sync custom presets when preferences load
  if (configInitialized && preferences.customPresets && customPresets.length === 0 && preferences.customPresets.length > 0) {
    setCustomPresets(preferences.customPresets);
  }
  if (configInitialized && preferences.hiddenPresets && hiddenPresets.length === 0 && preferences.hiddenPresets.length > 0) {
    setHiddenPresets(preferences.hiddenPresets);
  }

  const handleSavePreset = useCallback(async (preset: ConfigPreset) => {
    try {
      const updated = await saveCustomPreset(preset);
      setCustomPresets(updated);
    } catch (err) {
      console.error('Failed to save preset:', err);
    }
  }, []);

  const handleDeletePreset = useCallback(async (name: string) => {
    try {
      const updated = await deleteCustomPreset(name);
      setCustomPresets(updated);
      // If it was a builtin, track it as hidden locally
      setHiddenPresets((prev) => prev.includes(name) ? prev : [...prev, name]);
    } catch (err) {
      console.error('Failed to delete preset:', err);
    }
  }, []);

  const handleSavePreferences = useCallback(
    (prefs: UserPreferences) => {
      void savePreferences(prefs);
    },
    [savePreferences],
  );

  return (
    <Layout
      disks={disks}
      disksLoading={disksLoading}
      selectedDisk={selectedDisk}
      vmStatuses={vmStatuses}
      config={config}
      pricing={pricing}
      pricingLoading={pricingLoading}
      preferences={preferences}
      actionError={actionError}
      customPresets={customPresets}
      hiddenPresets={hiddenPresets}
      onSelectDisk={handleSelectDisk}
      onRefreshDisks={() => void refreshDisks()}
      onConfigChange={setConfig}
      onStartVm={() => void handleStartVm()}
      onStopVm={() => void handleStopVm()}
      onSavePreferences={handleSavePreferences}
      onSavePreset={(preset: ConfigPreset) => void handleSavePreset(preset)}
      onDeletePreset={(name: string) => void handleDeletePreset(name)}
    />
  );
}
