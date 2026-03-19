import { useState, useEffect, useCallback } from 'react';
import type { UserPreferences } from '../lib/types.ts';
import { getPreferences, setPreferences } from '../lib/tauri.ts';

const DEFAULT_PREFERENCES: UserPreferences = {
  project: '',
  zone: 'us-central1-a',
  defaultMachineType: 'n1-standard-8',
  defaultGpuType: 'nvidia-tesla-t4',
  defaultGpuCount: 4,
  defaultSpot: true,
  serviceAccountKeyPath: null,
  executionMode: 'gcloud',
  apiAccessToken: null,
  customPresets: [],
  hiddenPresets: [],
  diskConfigs: {},
};

export function useConfig() {
  const [preferences, setPrefs] = useState<UserPreferences>(DEFAULT_PREFERENCES);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    void (async () => {
      setLoading(true);
      try {
        const prefs = await getPreferences();
        setPrefs(prefs);
      } catch {
        // Use defaults if backend unavailable
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  const save = useCallback(async (updated: UserPreferences) => {
    try {
      const saved = await setPreferences(updated);
      setPrefs(saved);
    } catch {
      // Save failed; local state unchanged
    }
  }, []);

  return { preferences, loading, save };
}
