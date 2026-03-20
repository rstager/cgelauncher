import type { VmStatusUpdate } from '../lib/types.ts';

interface ResourceSummaryProps {
  vmStatus: VmStatusUpdate;
  costPerHour: number | null;
  spot: boolean;
}

export default function ResourceSummary({ vmStatus, costPerHour, spot }: ResourceSummaryProps) {
  const chips: { label: string; value: string }[] = [];

  if (vmStatus.machineType) {
    // Extract vCPU count from machine type name (e.g., "n1-standard-8" -> "8")
    const cpuMatch = vmStatus.machineType.match(/(\d+)$/);
    if (cpuMatch) {
      chips.push({ label: 'CPU', value: `${cpuMatch[1]} vCPU` });
    }
  }

  if (vmStatus.memoryGb) {
    chips.push({ label: 'Memory', value: `${vmStatus.memoryGb} GB` });
  }

  if (vmStatus.gpuType && vmStatus.gpuCount) {
    const gpuShort = formatGpuName(vmStatus.gpuType);
    chips.push({ label: 'GPU', value: `${vmStatus.gpuCount}x ${gpuShort}` });
  }

  chips.push({ label: 'Provisioning', value: spot ? 'Spot' : 'On-Demand' });

  if (costPerHour !== null) {
    chips.push({ label: 'Cost', value: `$${costPerHour.toFixed(2)}/hr` });
  }

  return (
    <div className="bg-[var(--color-bg-input)] border border-[var(--color-border-default)] rounded-lg p-3.5 mb-5">
      <h3 className="text-[13px] text-[var(--color-text-muted)] uppercase tracking-wider mb-2.5">
        Running Instance
      </h3>
      <div className="flex gap-2.5 flex-wrap">
        {chips.map((chip) => (
          <div
            key={chip.label}
            className="bg-[var(--color-bg-panel)] border border-[var(--color-border-default)] rounded-md px-3 py-1.5 text-xs"
          >
            <span className="text-[var(--color-text-muted)]">{chip.label}</span>
            <span className="text-[var(--color-text-primary)] font-semibold ml-1">{chip.value}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function formatGpuName(gpuType: string): string {
  // "nvidia-tesla-t4" -> "NVIDIA T4"
  const parts = gpuType.split('-');
  const model = parts[parts.length - 1].toUpperCase();
  return `NVIDIA ${model}`;
}
