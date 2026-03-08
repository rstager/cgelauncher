import type { PricingEstimate } from '../lib/types.ts';

interface PricingDisplayProps {
  pricing: PricingEstimate | null;
  loading: boolean;
}

function formatMonthly(hourly: number): string {
  const monthly = hourly * 730;
  return `~$${monthly.toLocaleString(undefined, { maximumFractionDigits: 0 })}/mo (730 hrs)`;
}

export default function PricingDisplay({ pricing, loading }: PricingDisplayProps) {
  if (!pricing && !loading) return null;

  return (
    <div className="bg-[var(--color-bg-panel)] border border-[var(--color-border-default)] rounded-lg p-4 mb-5">
      <h3 className="text-[13px] font-semibold text-[var(--color-text-muted)] uppercase tracking-wider mb-3">
        Estimated Cost
      </h3>

      {loading && !pricing ? (
        <div className="text-sm text-[var(--color-text-muted)]">Calculating...</div>
      ) : pricing ? (
        <>
          <div className="grid grid-cols-2 gap-3 mb-3">
            <div className="price-card price-card-spot">
              <div className="text-[11px] text-[var(--color-accent-green)] mb-0.5">Spot Pricing</div>
              <div className="text-xl font-bold text-[var(--color-text-primary)]">
                ${pricing.spotHourly.toFixed(2)}
                <span className="text-sm font-normal">/hr</span>
              </div>
              <div className="text-[11px] text-[var(--color-text-muted)]">
                {formatMonthly(pricing.spotHourly)}
              </div>
            </div>
            <div className="price-card">
              <div className="text-[11px] text-[var(--color-text-muted)] mb-0.5">On-Demand Pricing</div>
              <div className="text-xl font-bold text-[var(--color-text-primary)]">
                ${pricing.ondemandHourly.toFixed(2)}
                <span className="text-sm font-normal">/hr</span>
              </div>
              <div className="text-[11px] text-[var(--color-text-muted)]">
                {formatMonthly(pricing.ondemandHourly)}
              </div>
            </div>
          </div>

          {pricing.breakdown.length > 0 && (
            <div className="border-t border-[var(--color-border-muted)] pt-2.5">
              {pricing.breakdown.map((item) => (
                <div
                  key={item.label}
                  className="flex justify-between text-xs text-[var(--color-text-muted)] mb-1"
                >
                  <span>{item.label}</span>
                  <span className="text-[var(--color-text-secondary)]">
                    ${item.spotCost.toFixed(2)} / ${item.ondemandCost.toFixed(2)}
                  </span>
                </div>
              ))}
            </div>
          )}
        </>
      ) : null}
    </div>
  );
}
