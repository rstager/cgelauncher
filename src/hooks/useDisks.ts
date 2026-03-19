import { useState, useEffect, useCallback } from 'react';
import type { Disk } from '../lib/types.ts';
import { listDisks } from '../lib/tauri.ts';

export function useDisks(enabled = true) {
  const [disks, setDisks] = useState<Disk[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await listDisks();
      setDisks(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (enabled) {
      void refresh();
    } else {
      setLoading(false);
    }
  }, [refresh, enabled]);

  return { disks, loading, error, refresh };
}
