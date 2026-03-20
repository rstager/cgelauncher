import { useState } from 'react';
import type { Disk, VmStatusUpdate } from '../lib/types.ts';
import VmStatusBadge from './VmStatusBadge.tsx';
import { usePricing } from '../hooks/usePricing.ts';

interface DiskItemProps {
  disk: Disk;
  selected: boolean;
  vmStatus: VmStatusUpdate | undefined;
  onSelect: () => void;
  onDelete: () => void;
}

export default function DiskItem({ disk, selected, vmStatus, onSelect, onDelete }: DiskItemProps) {
  const diskReady = disk.status === 'READY';
  const effectiveStatus = vmStatus?.status ?? (disk.attachedTo ? 'Running' : 'Stopped');
  const isRunning = effectiveStatus === 'Running';

  const { pricing } = usePricing(isRunning ? {
    machineType: vmStatus?.machineType ?? '',
    gpuType: vmStatus?.gpuType ?? null,
    gpuCount: vmStatus?.gpuCount ?? null,
    spot: true,
  } : { machineType: '', gpuType: null, gpuCount: null, spot: true });
  const isTransitioning = effectiveStatus === 'Starting' || effectiveStatus === 'Stopping';
  const isStopped = effectiveStatus === 'Stopped' || effectiveStatus === 'NotFound';
  const vmName = vmStatus?.instanceName ?? disk.attachedTo ?? disk.name;
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [confirmInput, setConfirmInput] = useState('');

  function handleDelete(e: React.MouseEvent) {
    e.stopPropagation();
    if (!canDelete) return;
    setConfirmInput('');
    setConfirmOpen(true);
  }

  function handleConfirmDelete(e: React.MouseEvent) {
    e.stopPropagation();
    if (confirmInput === disk.name) {
      setConfirmOpen(false);
      onDelete();
    }
  }

  function handleCancelDelete(e: React.MouseEvent) {
    e.stopPropagation();
    setConfirmOpen(false);
  }

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

      if (pricing?.spotHourly != null) {
        parts.push(`~$${pricing.spotHourly.toFixed(2)}/hr`);
      }

      return (
        <span className="text-[var(--color-text-success)]">
          {vmName}
          {parts.length > 0 ? ` - ${parts.join(', ')}` : ''}
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

    if (!diskReady) {
      return <span className="text-[var(--color-text-muted)] italic">{disk.status.toLowerCase()}...</span>;
    }
    return <span>Stopped &middot; {disk.sizeGb} GB {disk.type} &middot; ~${diskMonthlyCost(disk.sizeGb, disk.type)}/mo</span>;
  }

  const canDelete = diskReady && isStopped;

  return (
    <>
    {confirmOpen && (
      <div className="fixed inset-0 bg-black/50 z-50 flex items-center justify-center" onClick={handleCancelDelete}>
        <div className="bg-[var(--color-bg-panel)] border border-[var(--color-border-default)] rounded-lg p-6 w-80 shadow-xl" onClick={e => e.stopPropagation()}>
          <h2 className="text-sm font-semibold text-[var(--color-text-primary)] mb-2">Delete disk</h2>
          <p className="text-xs text-[var(--color-text-muted)] mb-4">
            This is permanent and cannot be undone. Type <span className="font-mono text-[var(--color-accent-red)]">{disk.name}</span> to confirm.
          </p>
          <input
            autoFocus
            type="text"
            className="w-full bg-[var(--color-bg-input)] border border-[var(--color-border-default)] text-[var(--color-text-primary)] px-3 py-1.5 rounded text-sm mb-4 outline-none focus:border-[var(--color-accent-red)]"
            placeholder={disk.name}
            value={confirmInput}
            onChange={e => setConfirmInput(e.target.value)}
            onKeyDown={e => { if (e.key === 'Enter') handleConfirmDelete(e as unknown as React.MouseEvent); if (e.key === 'Escape') setConfirmOpen(false); }}
          />
          <div className="flex gap-2 justify-end">
            <button
              className="bg-transparent border border-[var(--color-border-default)] text-[var(--color-text-muted)] px-3 py-1.5 rounded text-xs cursor-pointer hover:border-[var(--color-text-secondary)]"
              onClick={handleCancelDelete}
            >
              Cancel
            </button>
            <button
              className="border-none px-3 py-1.5 rounded text-xs font-semibold transition-opacity"
              style={{ background: 'var(--color-accent-red)', color: '#fff', opacity: confirmInput === disk.name ? 1 : 0.4, cursor: confirmInput === disk.name ? 'pointer' : 'not-allowed' }}
              onClick={handleConfirmDelete}
            >
              Delete
            </button>
          </div>
        </div>
      </div>
    )}

    <div
      className={`disk-item ${selected ? 'disk-item-selected' : ''} ${!diskReady ? 'opacity-50 cursor-not-allowed' : ''}`}
      onClick={diskReady ? onSelect : undefined}
    >
      <div className="flex items-center gap-2 mb-1">
        <VmStatusBadge status={effectiveStatus} />
        <span className="text-sm font-medium text-[var(--color-text-primary)] flex-1">{disk.name}</span>
        {canDelete && (
          <button
            className="bg-transparent border-none text-[var(--color-text-muted)] cursor-pointer text-[11px] px-1 py-0.5 rounded opacity-0 group-hover:opacity-100 hover:text-[var(--color-accent-red)] disk-item-delete"
            title="Delete disk"
            onClick={handleDelete}
          >
            ✕
          </button>
        )}
      </div>
      <div className="text-[11px] text-[var(--color-text-muted)] ml-[18px]">
        {renderMeta()}
      </div>
    </div>
    </>
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

// GCP persistent disk storage rates ($/GB/month, us-central1)
const DISK_COST_PER_GB: Record<string, number> = {
  'pd-standard': 0.040,
  'pd-balanced': 0.100,
  'pd-ssd':      0.170,
};

function diskMonthlyCost(sizeGb: number, diskType: string): string {
  const rate = DISK_COST_PER_GB[diskType] ?? 0.100;
  const cost = sizeGb * rate;
  return cost >= 10 ? cost.toFixed(0) : cost.toFixed(2);
}
