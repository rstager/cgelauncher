import { useState, useMemo } from 'react';
import type { MachineConfig as MachineConfigType, ConfigPreset } from '../lib/types.ts';

const BUILTIN_PRESETS: ConfigPreset[] = [
  {
    name: 'Inference / Dev',
    machineType: 'g2-standard-4',
    gpuType: null,
    gpuCount: null,
    description: '4 vCPU · 16 GB · 1x L4',
  },
  {
    name: 'ML Training',
    machineType: 'n1-standard-8',
    gpuType: 'nvidia-tesla-t4',
    gpuCount: 4,
    description: '8 vCPU \u00b7 30 GB \u00b7 4x T4',
  },
  {
    name: 'A100 Training',
    machineType: 'a2-highgpu-1g',
    gpuType: null,
    gpuCount: null,
    description: '12 vCPU \u00b7 85 GB \u00b7 1x A100 40GB',
  },
  {
    name: 'CPU Only',
    machineType: 'n1-standard-8',
    gpuType: null,
    gpuCount: null,
    description: '8 vCPU \u00b7 30 GB \u00b7 No GPU',
  },
];

const MACHINE_TYPES = [
  'n1-standard-4',
  'n1-standard-8',
  'n1-standard-16',
  'n1-standard-32',
  'n1-highmem-8',
  'n1-highmem-16',
  'g2-standard-4',
  'g2-standard-8',
  'g2-standard-16',
  'a2-highgpu-1g',
  'a2-highgpu-2g',
];

const GPU_TYPES = [
  { value: '', label: 'None' },
  { value: 'nvidia-tesla-t4', label: 'NVIDIA Tesla T4' },
  { value: 'nvidia-tesla-a100', label: 'NVIDIA Tesla A100' },
  { value: 'nvidia-tesla-v100', label: 'NVIDIA Tesla V100' },
  { value: 'nvidia-tesla-p100', label: 'NVIDIA Tesla P100' },
];

const GPU_COUNTS = [1, 2, 4, 8];

const PRESET_PRICES: Record<string, string> = {
  'Inference / Dev': '~$0.23/hr spot',
  'ML Training': '~$0.56/hr spot',
  'A100 Training': '~$1.10/hr spot',
  'CPU Only': '~$0.08/hr spot',
};

function buildDescription(config: MachineConfigType): string {
  const parts: string[] = [config.machineType];
  if (config.gpuType && config.gpuCount) {
    const gpuShort = config.gpuType.split('-').pop()?.toUpperCase() ?? config.gpuType;
    parts.push(`${config.gpuCount}x ${gpuShort}`);
  } else {
    parts.push('No GPU');
  }
  return parts.join(' \u00b7 ');
}

interface MachineConfigProps {
  config: MachineConfigType;
  customPresets: ConfigPreset[];
  hiddenPresets: string[];
  onChange: (config: MachineConfigType) => void;
  onSavePreset: (preset: ConfigPreset) => void;
  onDeletePreset: (name: string) => void;
}

export default function MachineConfig({ config, customPresets, hiddenPresets, onChange, onSavePreset, onDeletePreset }: MachineConfigProps) {
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const [savePresetOpen, setSavePresetOpen] = useState(false);
  const [presetName, setPresetName] = useState('');

  const allPresets = useMemo(() => [
    ...BUILTIN_PRESETS.filter((p) => !hiddenPresets.includes(p.name)),
    ...customPresets,
  ], [customPresets, hiddenPresets]);
  const builtinNames = useMemo(() => new Set(BUILTIN_PRESETS.map((p) => p.name)), []);

  const selectedPreset = useMemo(() => {
    return allPresets.find(
      (p) =>
        p.machineType === config.machineType &&
        p.gpuType === (config.gpuType ?? null) &&
        p.gpuCount === (config.gpuCount ?? null),
    );
  }, [config.machineType, config.gpuType, config.gpuCount, allPresets]);

  function selectPreset(preset: ConfigPreset) {
    onChange({
      ...config,
      machineType: preset.machineType,
      gpuType: preset.gpuType,
      gpuCount: preset.gpuCount,
    });
  }

  function handleSavePreset() {
    const name = presetName.trim();
    if (!name) return;
    onSavePreset({
      name,
      machineType: config.machineType,
      gpuType: config.gpuType,
      gpuCount: config.gpuCount,
      description: buildDescription(config),
    });
    setPresetName('');
    setSavePresetOpen(false);
  }

  return (
    <div>
      <div className="text-xs font-semibold text-[var(--color-text-muted)] uppercase tracking-wider mb-2.5 flex items-center gap-2">
        <span>Quick Presets</span>
        <button
          className="text-[11px] font-normal normal-case tracking-normal text-[var(--color-text-link)] hover:underline cursor-pointer bg-transparent border-none"
          onClick={() => setSavePresetOpen(!savePresetOpen)}
        >
          + Save Current
        </button>
      </div>

      {savePresetOpen && (
        <div className="flex items-center gap-2 mb-3">
          <input
            type="text"
            className="input-field flex-1 max-w-[200px]"
            placeholder="Preset name"
            value={presetName}
            onChange={(e) => setPresetName(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSavePreset()}
            autoFocus
          />
          <button
            className="text-xs px-3 py-1.5 rounded bg-[var(--color-accent-green-dark)] text-white border-none cursor-pointer hover:bg-[var(--color-accent-green-hover)]"
            onClick={handleSavePreset}
            disabled={!presetName.trim()}
          >
            Save
          </button>
          <button
            className="text-xs px-3 py-1.5 rounded bg-transparent border border-[var(--color-border-default)] text-[var(--color-text-muted)] cursor-pointer hover:text-[var(--color-text-secondary)]"
            onClick={() => { setSavePresetOpen(false); setPresetName(''); }}
          >
            Cancel
          </button>
        </div>
      )}

      <div className="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-2.5 mb-5">
        {allPresets.map((preset) => (
          <div
            key={preset.name}
            className={`preset-card relative ${selectedPreset?.name === preset.name ? 'preset-card-selected' : ''}`}
            onClick={() => selectPreset(preset)}
          >
            <div className="text-[13px] font-semibold text-[var(--color-text-primary)] mb-1 flex items-center gap-1">
              {preset.name}
              {!builtinNames.has(preset.name) && (
                <span className="text-[9px] px-1 py-0.5 rounded bg-[var(--color-border-default)] text-[var(--color-text-muted)] font-normal">
                  custom
                </span>
              )}
            </div>
            <div className="text-[11px] text-[var(--color-text-muted)] leading-relaxed">
              {preset.description}
            </div>
            {PRESET_PRICES[preset.name] && (
              <div className="text-xs text-[var(--color-text-warning)] mt-1.5">
                {PRESET_PRICES[preset.name]}
              </div>
            )}
            <button
              className="absolute top-2 right-2 text-[var(--color-text-muted)] hover:text-[var(--color-accent-red)] bg-transparent border-none cursor-pointer text-xs"
              onClick={(e) => { e.stopPropagation(); onDeletePreset(preset.name); }}
              title="Delete preset"
            >
              &times;
            </button>
          </div>
        ))}
      </div>

      <button
        className="bg-transparent border-none text-[var(--color-text-secondary)] text-[13px] font-medium cursor-pointer py-1.5 mb-3 flex items-center gap-1 hover:text-[var(--color-text-link)] hover:underline"
        onClick={() => setAdvancedOpen(!advancedOpen)}
      >
        <span>{advancedOpen ? '\u25BC' : '\u25B6'}</span>
        &nbsp; Advanced Configuration
      </button>

      {advancedOpen && (
        <div className="bg-[var(--color-bg-panel)] border border-[var(--color-border-default)] rounded-lg p-4 mb-5">
          <div className="flex items-center gap-3 mb-3">
            <label className="text-[13px] text-[var(--color-text-secondary)] w-[110px] shrink-0">
              Machine Type
            </label>
            <select
              className="select-field flex-1 max-w-[260px]"
              value={config.machineType}
              onChange={(e) => onChange({ ...config, machineType: e.target.value })}
            >
              {MACHINE_TYPES.map((mt) => (
                <option key={mt} value={mt}>{mt}</option>
              ))}
            </select>
          </div>
          <div className="flex items-center gap-3 mb-3">
            <label className="text-[13px] text-[var(--color-text-secondary)] w-[110px] shrink-0">
              GPU Type
            </label>
            <select
              className="select-field flex-1 max-w-[260px]"
              value={config.gpuType ?? ''}
              onChange={(e) => onChange({
                ...config,
                gpuType: e.target.value || null,
                gpuCount: e.target.value ? (config.gpuCount ?? 1) : null,
              })}
            >
              {GPU_TYPES.map((gt) => (
                <option key={gt.value} value={gt.value}>{gt.label}</option>
              ))}
            </select>
          </div>
          {config.gpuType && (
            <div className="flex items-center gap-3 mb-3">
              <label className="text-[13px] text-[var(--color-text-secondary)] w-[110px] shrink-0">
                GPU Count
              </label>
              <select
                className="select-field flex-1 max-w-[260px]"
                value={config.gpuCount ?? 1}
                onChange={(e) => onChange({ ...config, gpuCount: Number(e.target.value) })}
              >
                {GPU_COUNTS.map((c) => (
                  <option key={c} value={c}>{c}</option>
                ))}
              </select>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
