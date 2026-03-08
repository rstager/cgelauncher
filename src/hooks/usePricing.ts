import { useState, useEffect, useRef } from 'react';
import type { MachineConfig, PricingEstimate } from '../lib/types.ts';
import { estimatePricing } from '../lib/tauri.ts';

export function usePricing(config: MachineConfig) {
  const [pricing, setPricing] = useState<PricingEstimate | null>(null);
  const [loading, setLoading] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
    }

    timerRef.current = setTimeout(async () => {
      setLoading(true);
      try {
        const result = await estimatePricing(config);
        setPricing(result);
      } catch {
        // Pricing fetch failed; retain previous estimate
      } finally {
        setLoading(false);
      }
    }, 300);

    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
    };
  }, [config.machineType, config.gpuType, config.gpuCount, config.spot]);

  return { pricing, loading };
}
