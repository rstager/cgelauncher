import type { Disk, VmStatusUpdate } from '../lib/types.ts';
import VmStatusBadge from './VmStatusBadge.tsx';

interface DiskItemProps {
  disk: Disk;
  selected: boolean;
  vmStatus: VmStatusUpdate | undefined;
  onSelect: () => void;
}

export default function DiskItem({ disk, selected, vmStatus, onSelect }: DiskItemProps) {
  const isRunning = vmStatus?.status === 'Running' || (vmStatus == null && disk.attachedTo != null);
  const isTransitioning = vmStatus?.status === 'Starting' || vmStatus?.status === 'Stopping';
  const effectiveStatus = vmStatus?.status ?? (disk.attachedTo ? 'Running' : 'Stopped');
  const vmName = vmStatus?.instanceName ?? disk.attachedTo ?? disk.name;

  function renderMeta() {
    if (isTransitioning) {
      return <span>{vmStatus!.status}...</span>;
    }

    if (isRunning && vmStatus) {
      const parts: string[] = [];
      const cpuCount = cpuCountFromMachineType(vmStatus.machineType);

      if (cpuCount != null) parts.push(`${cpuCount}xCPU`);
      if (vmStatus.memoryGb) parts.push(`${formatMemoryGb(vmStatus.memoryGb)}GB`);
      if (vmStatus.gpuType && vmStatus.gpuCount) {
        parts.push(`${vmStatus.gpuCount}x${formatGpuShort(vmStatus.gpuType)}`);
      }

      if (parts.length === 0 && vmStatus.machineType) {
        parts.push(vmStatus.machineType);
      }

      return (
        <span className="text-[var(--color-text-success)]">
          {vmName}
          {parts.length > 0 ? ` - ${parts.join(',')}` : ''}
        </span>
      );
    }

    if (isRunning) {
      return (
        <span className="text-[var(--color-text-success)]">
          {vmName}
        </span>
      );
    }

    return <span>Stopped &middot; {disk.sizeGb} GB {disk.type}</span>;
  }

  return (
    <div
      className={`disk-item ${selected ? 'disk-item-selected' : ''}`}
      onClick={onSelect}
    >
      <div className="flex items-center gap-2 mb-1">
        <VmStatusBadge status={effectiveStatus} />
        <span className="text-sm font-medium text-[var(--color-text-primary)]">{disk.name}</span>
      </div>
      <div className="text-[11px] text-[var(--color-text-muted)] ml-[18px]">
        {renderMeta()}
      </div>
    </div>
  );
}

function formatGpuShort(gpuType: string): string {
  // "nvidia-tesla-t4" -> "T4"
  const parts = gpuType.split('-');
  return parts[parts.length - 1].toUpperCase();
}

function cpuCountFromMachineType(machineType: string | null): number | null {
  if (!machineType) {
    return null;
  }

  const match = machineType.match(/-(\d+)$/);
  if (!match) {
    return null;
  }

  const cpuCount = Number.parseInt(match[1], 10);
  return Number.isNaN(cpuCount) ? null : cpuCount;
}

function formatMemoryGb(memoryGb: number): string {
  return Number.isInteger(memoryGb) ? String(memoryGb) : memoryGb.toFixed(1);
}
