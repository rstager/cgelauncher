import { launchSshTerminal } from '../lib/tauri.ts';
import type { VmStatus } from '../lib/types.ts';

interface VmControlsProps {
  vmStatus: VmStatus;
  diskReady: boolean;
  instanceName: string | null;
  onStart: () => void;
  onStop: () => void;
}

export default function VmControls({ vmStatus, diskReady, instanceName, onStart, onStop }: VmControlsProps) {
  const isRunning = vmStatus === 'Running';
  const isStopped = vmStatus === 'Stopped' || vmStatus === 'NotFound';
  const isTransitioning = vmStatus === 'Starting' || vmStatus === 'Stopping';

  function handleSsh() {
    if (instanceName) {
      launchSshTerminal(instanceName).catch(console.error);
    }
  }

  return (
    <div className="mb-6">
      <div className="flex gap-3 items-center mb-2">
        <button
          className="btn-action btn-start"
          disabled={!diskReady || !isStopped}
          onClick={onStart}
          title={!diskReady ? 'Disk is not ready' : undefined}
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
      </div>
      <div className="flex items-center">
        {isRunning && instanceName && (
          <button className="btn-action btn-ssh ml-auto" onClick={handleSsh}>
            SSH
          </button>
        )}
      </div>
    </div>
  );
}
