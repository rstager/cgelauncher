import type { VmStatus } from '../lib/types.ts';

interface VmControlsProps {
  vmStatus: VmStatus;
  onStart: () => void;
  onStop: () => void;
}

export default function VmControls({ vmStatus, onStart, onStop }: VmControlsProps) {
  const isRunning = vmStatus === 'Running';
  const isStopped = vmStatus === 'Stopped' || vmStatus === 'NotFound';
  const isTransitioning = vmStatus === 'Starting' || vmStatus === 'Stopping';

  return (
    <div className="flex gap-3 items-center mb-6">
      <button
        className="btn-action btn-start"
        disabled={isRunning || isTransitioning}
        onClick={onStart}
      >
        {vmStatus === 'Starting' ? 'Starting...' : 'Start VM'}
      </button>
      <button
        className="btn-action btn-stop"
        disabled={isStopped || isTransitioning}
        onClick={onStop}
      >
        {vmStatus === 'Stopping' ? 'Stopping...' : 'Stop VM'}
      </button>
      <span className="text-xs text-[var(--color-text-muted)]">
        Will delete VM, disk is preserved
      </span>
    </div>
  );
}
