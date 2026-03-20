import type { Disk, VmStatusUpdate } from '../lib/types.ts';
import DiskItem from './DiskItem.tsx';

interface DiskListProps {
  disks: Disk[];
  selectedDisk: string | null;
  vmStatuses: Map<string, VmStatusUpdate>;
  loading: boolean;
  onSelectDisk: (name: string) => void;
  onRefresh: () => void;
  onCreateDisk: () => void;
  onDeleteDisk: (name: string) => void;
}

export default function DiskList({
  disks,
  selectedDisk,
  vmStatuses,
  loading,
  onSelectDisk,
  onRefresh,
  onCreateDisk,
  onDeleteDisk,
}: DiskListProps) {
  return (
    <div className="w-[300px] min-w-[260px] bg-[var(--color-bg-panel)] border-r border-[var(--color-border-default)] flex flex-col">
      <div className="px-4 py-3 border-b border-[var(--color-border-default)] flex items-center justify-between">
        <h2 className="text-[13px] font-semibold text-[var(--color-text-muted)] uppercase tracking-wider">
          Disks
        </h2>
        <div className="flex items-center gap-1">
          <button
            className="bg-transparent border-none text-[var(--color-text-muted)] cursor-pointer text-base px-1.5 py-0.5 rounded hover:bg-[var(--color-border-default)] hover:text-[var(--color-text-secondary)]"
            title="Create Disk"
            onClick={onCreateDisk}
          >
            +
          </button>
          <button
            className="bg-transparent border-none text-[var(--color-text-muted)] cursor-pointer text-base px-1.5 py-0.5 rounded hover:bg-[var(--color-border-default)] hover:text-[var(--color-text-secondary)]"
            title="Refresh"
            onClick={onRefresh}
          >
            &#x21bb;
          </button>
        </div>
      </div>
      <div className="flex-1 overflow-y-auto">
        {loading && disks.length === 0 ? (
          <div className="p-4 text-sm text-[var(--color-text-muted)]">Loading disks...</div>
        ) : (
          disks.map((disk) => (
            <DiskItem
              key={disk.name}
              disk={disk}
              selected={disk.name === selectedDisk}
              vmStatus={vmStatuses.get(disk.name)}
              onSelect={() => onSelectDisk(disk.name)}
              onDelete={() => onDeleteDisk(disk.name)}
            />
          ))
        )}
      </div>
      {disks.some((d) => d.status === 'READY' && d.attachedTo == null) && (
        <div className="px-4 py-2 text-[10px] text-[var(--color-text-muted)] opacity-60 border-t border-[var(--color-border-default)]">
          * Disk cost: us-central1 rate, updated 2025-03-19
        </div>
      )}
    </div>
  );
}
