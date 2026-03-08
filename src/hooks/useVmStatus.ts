import { useState, useEffect, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import type { VmStatusUpdate } from '../lib/types.ts';

export function useVmStatus() {
  const [statuses, setStatuses] = useState<Map<string, VmStatusUpdate>>(new Map());

  useEffect(() => {
    const unlisten = listen<VmStatusUpdate>('vm-status-update', (event) => {
      setStatuses((prev) => {
        const next = new Map(prev);
        next.set(event.payload.diskName, event.payload);
        return next;
      });
    });

    return () => {
      void unlisten.then((fn) => fn());
    };
  }, []);

  const getStatus = useCallback(
    (diskName: string): VmStatusUpdate | undefined => {
      return statuses.get(diskName);
    },
    [statuses],
  );

  const upsertStatus = useCallback((update: VmStatusUpdate) => {
    setStatuses((prev) => {
      const next = new Map(prev);
      next.set(update.diskName, update);
      return next;
    });
  }, []);

  return { statuses, getStatus, upsertStatus };
}
