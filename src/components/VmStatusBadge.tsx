import type { VmStatus } from '../lib/types.ts';

interface VmStatusBadgeProps {
  status: VmStatus;
}

function dotClass(status: VmStatus): string {
  switch (status) {
    case 'Running':
      return 'status-dot status-dot-running';
    case 'Starting':
    case 'Stopping':
      return 'status-dot status-dot-transitioning';
    case 'Stopped':
    case 'NotFound':
      return 'status-dot status-dot-stopped';
  }
}

export default function VmStatusBadge({ status }: VmStatusBadgeProps) {
  return <div className={dotClass(status)} />;
}
