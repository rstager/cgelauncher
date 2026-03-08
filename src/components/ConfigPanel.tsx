import { useMemo } from 'react';
import type { Disk, MachineConfig as MachineConfigType, ConfigPreset, VmStatusUpdate, PricingEstimate } from '../lib/types.ts';
import ResourceSummary from './ResourceSummary.tsx';
import VmControls from './VmControls.tsx';
import MachineConfig from './MachineConfig.tsx';
import PricingDisplay from './PricingDisplay.tsx';

interface ConfigPanelProps {
  disk: Disk;
  vmStatus: VmStatusUpdate | undefined;
  config: MachineConfigType;
  pricing: PricingEstimate | null;
  pricingLoading: boolean;
  customPresets: ConfigPreset[];
  hiddenPresets: string[];
  onConfigChange: (config: MachineConfigType) => void;
  onStart: () => void;
  onStop: () => void;
  onSavePreset: (preset: ConfigPreset) => void;
  onDeletePreset: (name: string) => void;
}

export default function ConfigPanel({
  disk,
  vmStatus,
  config,
  pricing,
  pricingLoading,
  customPresets,
  hiddenPresets,
  onConfigChange,
  onStart,
  onStop,
  onSavePreset,
  onDeletePreset,
}: ConfigPanelProps) {
  const effectiveStatus = vmStatus?.status ?? (disk.attachedTo ? 'Running' : 'Stopped');
  const vmName = disk.name;
  const isRunning = effectiveStatus === 'Running';

  const spotCostPerHour = useMemo(() => {
    return pricing?.spotHourly ?? null;
  }, [pricing]);

  return (
    <div className="flex-1 p-6 overflow-y-auto">
      <h2 className="text-lg font-semibold mb-1">{disk.name}</h2>
      <div className="text-[13px] text-[var(--color-text-muted)] mb-5">
        VM: {vmName} &middot; {effectiveStatus}
      </div>

      {isRunning && vmStatus && (
        <ResourceSummary vmStatus={vmStatus} costPerHour={spotCostPerHour} />
      )}

      <VmControls vmStatus={effectiveStatus} onStart={onStart} onStop={onStop} />

      <hr className="border-none border-t border-[var(--color-border-default)] mb-6" style={{ borderTop: '1px solid var(--color-border-default)' }} />

      <h2 className="text-sm text-[var(--color-text-muted)] font-semibold mb-4">
        Configuration (for next launch)
      </h2>

      <MachineConfig
        config={config}
        customPresets={customPresets}
        hiddenPresets={hiddenPresets}
        onChange={onConfigChange}
        onSavePreset={onSavePreset}
        onDeletePreset={onDeletePreset}
      />

      <div className="text-xs font-semibold text-[var(--color-text-muted)] uppercase tracking-wider mb-2.5">
        Provisioning
      </div>
      <div className="flex mb-5">
        <button
          className={`toggle-btn ${config.spot ? 'toggle-btn-active' : ''}`}
          onClick={() => onConfigChange({ ...config, spot: true })}
        >
          Spot (up to 91% off)
        </button>
        <button
          className={`toggle-btn ${!config.spot ? 'toggle-btn-active-alt' : ''}`}
          onClick={() => onConfigChange({ ...config, spot: false })}
        >
          On-Demand
        </button>
      </div>

      <PricingDisplay pricing={pricing} loading={pricingLoading} />
    </div>
  );
}
