import { useState, useEffect, useRef } from 'react';
import type {
  Disk,
  MachineConfig,
  ConfigPreset,
  VmStatusUpdate,
  UserPreferences,
  PricingEstimate,
  GcloudCommandLogEntry,
} from '../lib/types.ts';
import type { AuthStatus } from '../lib/types.ts';
import { checkAuth, getGcloudLogs } from '../lib/tauri.ts';
import DiskList from './DiskList.tsx';
import ConfigPanel from './ConfigPanel.tsx';
import StatusBar from './StatusBar.tsx';
import SettingsPanel from './SettingsPanel.tsx';

interface LayoutProps {
  disks: Disk[];
  disksLoading: boolean;
  selectedDisk: string | null;
  vmStatuses: Map<string, VmStatusUpdate>;
  config: MachineConfig;
  pricing: PricingEstimate | null;
  pricingLoading: boolean;
  preferences: UserPreferences;
  actionError: string | null;
  customPresets: ConfigPreset[];
  hiddenPresets: string[];
  onSelectDisk: (name: string) => void;
  onRefreshDisks: () => void;
  onConfigChange: (config: MachineConfig) => void;
  onStartVm: () => void;
  onStopVm: () => void;
  onSavePreferences: (prefs: UserPreferences) => void;
  onSavePreset: (preset: ConfigPreset) => void;
  onDeletePreset: (name: string) => void;
}

export default function Layout({
  disks,
  disksLoading,
  selectedDisk,
  vmStatuses,
  config,
  pricing,
  pricingLoading,
  preferences,
  actionError,
  customPresets,
  hiddenPresets,
  onSelectDisk,
  onRefreshDisks,
  onConfigChange,
  onStartVm,
  onStopVm,
  onSavePreferences,
  onSavePreset,
  onDeletePreset,
}: LayoutProps) {
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [logsOpen, setLogsOpen] = useState(false);
  const [logsLoading, setLogsLoading] = useState(false);
  const [logsError, setLogsError] = useState<string | null>(null);
  const [logs, setLogs] = useState<GcloudCommandLogEntry[]>([]);
  const [authStatus, setAuthStatus] = useState<AuthStatus | null>(null);
  const logsEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    void checkAuth().then(setAuthStatus).catch(() => {});
  }, []);

  const selectedDiskData = disks.find((d) => d.name === selectedDisk);
  const selectedVmStatus = selectedDisk ? vmStatuses.get(selectedDisk) : undefined;
  const isSelectedRunning =
    selectedVmStatus?.status === 'Running' ||
    (!selectedVmStatus && selectedDiskData?.attachedTo != null);

  const loadLogs = async () => {
    setLogsLoading(true);
    setLogsError(null);
    try {
      const entries = await getGcloudLogs();
      setLogs(entries.slice().reverse());
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setLogsError(message);
    } finally {
      setLogsLoading(false);
    }
  };

  const handleOpenLogs = () => {
    setLogsOpen(true);
    void loadLogs();
  };

  useEffect(() => {
    if (!logsOpen) {
      return;
    }

    const timer = setInterval(() => {
      void loadLogs();
    }, 1500);

    return () => clearInterval(timer);
  }, [logsOpen]);

  useEffect(() => {
    logsEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs]);

  return (
    <>
      {/* Top Bar */}
      <div className="bg-[var(--color-bg-panel)] border-b border-[var(--color-border-default)] px-5 py-3 flex items-center gap-4">
        <h1 className="text-base font-semibold text-[var(--color-text-primary)] mr-auto">
          GCE VM Launcher
        </h1>
        <label className="text-xs text-[var(--color-text-muted)]">Project</label>
        <input
          type="text"
          className="bg-[var(--color-bg-input)] border border-[var(--color-border-default)] text-[var(--color-text-secondary)] px-2 py-1 rounded text-[13px] w-40"
          value={preferences.project}
          readOnly
        />
        <label className="text-xs text-[var(--color-text-muted)]">Zone</label>
        <select
          className="bg-[var(--color-bg-input)] border border-[var(--color-border-default)] text-[var(--color-text-secondary)] px-2 py-1 rounded text-[13px] min-w-[140px]"
          value={preferences.zone}
          disabled
        >
          <option>{preferences.zone}</option>
        </select>
        {authStatus?.authenticated && (
          <span className="text-[11px] px-2 py-0.5 rounded-full bg-[var(--color-auth-bg)] text-[var(--color-text-success)]">
            {authStatus.method === 'gcloud' ? 'gcloud authenticated' : `SA: ${authStatus.account}`}
          </span>
        )}
        <button
          className="bg-transparent border border-[var(--color-border-default)] text-[var(--color-text-muted)] px-2.5 py-1 rounded text-[13px] cursor-pointer hover:border-[var(--color-text-link)] hover:text-[var(--color-text-link)]"
          onClick={() => handleOpenLogs()}
        >
          Log
        </button>
        <button
          className="bg-transparent border border-[var(--color-border-default)] text-[var(--color-text-muted)] px-2.5 py-1 rounded text-[13px] cursor-pointer hover:border-[var(--color-text-link)] hover:text-[var(--color-text-link)]"
          onClick={() => setSettingsOpen(true)}
        >
          Settings
        </button>
      </div>

      {/* Main Content */}
      <div className="flex flex-1 overflow-hidden">
        <DiskList
          disks={disks}
          selectedDisk={selectedDisk}
          vmStatuses={vmStatuses}
          loading={disksLoading}
          onSelectDisk={onSelectDisk}
          onRefresh={onRefreshDisks}
        />
        {selectedDiskData ? (
          <ConfigPanel
            disk={selectedDiskData}
            vmStatus={selectedVmStatus}
            config={config}
            pricing={pricing}
            pricingLoading={pricingLoading}
            customPresets={customPresets}
            hiddenPresets={hiddenPresets}
            onConfigChange={onConfigChange}
            onStart={onStartVm}
            onStop={onStopVm}
            onSavePreset={onSavePreset}
            onDeletePreset={onDeletePreset}
          />
        ) : (
          <div className="flex-1 flex items-center justify-center text-[var(--color-text-muted)]">
            Select a disk to configure
          </div>
        )}
      </div>

      {actionError && (
        <div className="px-5 py-2 text-xs text-[var(--color-accent-red)] border-t border-[var(--color-border-default)] bg-[var(--color-bg-panel)]">
          {actionError}
        </div>
      )}

      {/* Status Bar */}
      <StatusBar
        diskName={selectedDisk}
        project={preferences.project}
        zone={preferences.zone}
        isRunning={isSelectedRunning}
      />

      {/* Settings Modal */}
      {settingsOpen && (
        <SettingsPanel
          preferences={preferences}
          onSave={onSavePreferences}
          onClose={() => setSettingsOpen(false)}
        />
      )}

      {logsOpen && (
        <div className="fixed inset-0 bg-black/50 z-40 flex items-center justify-center p-6">
          <div className="bg-[var(--color-bg-panel)] border border-[var(--color-border-default)] rounded-lg w-full max-w-4xl max-h-[80vh] flex flex-col overflow-hidden">
            <div className="px-4 py-3 border-b border-[var(--color-border-default)] flex items-center gap-2">
              <h2 className="text-sm font-semibold text-[var(--color-text-primary)] mr-auto">gcloud log</h2>
              <button
                className="bg-transparent border border-[var(--color-border-default)] text-[var(--color-text-muted)] px-2.5 py-1 rounded text-[13px] cursor-pointer hover:border-[var(--color-text-link)] hover:text-[var(--color-text-link)]"
                onClick={() => void loadLogs()}
              >
                Refresh
              </button>
              <button
                className="bg-transparent border border-[var(--color-border-default)] text-[var(--color-text-muted)] px-2.5 py-1 rounded text-[13px] cursor-pointer hover:border-[var(--color-text-link)] hover:text-[var(--color-text-link)]"
                onClick={() => setLogsOpen(false)}
              >
                Close
              </button>
            </div>
            <div className="p-4 overflow-auto text-xs text-[var(--color-text-secondary)] space-y-3">
              {logsLoading && <div>Loading logs...</div>}
              {logsError && <div className="text-[var(--color-accent-red)]">{logsError}</div>}
              {!logsLoading && !logsError && logs.length === 0 && (
                <div className="text-[var(--color-text-muted)]">No gcloud commands have run yet.</div>
              )}
              {!logsLoading && !logsError && logs.map((entry, index) => (
                <div
                  key={`${entry.command}-${index}`}
                  className="border border-[var(--color-border-default)] rounded p-3 bg-[var(--color-bg-input)]"
                >
                  <div className="font-mono text-[11px] text-[var(--color-text-primary)] break-all">
                    {entry.command}
                  </div>
                  <div className={`mt-1 ${entry.success ? 'text-[var(--color-text-success)]' : 'text-[var(--color-accent-red)]'}`}>
                    {entry.success ? 'OK' : 'ERROR'} (exit {entry.exitCode})
                  </div>
                  <pre className="mt-2 whitespace-pre-wrap break-words text-[11px] text-[var(--color-text-secondary)]">
                    {entry.response || '(no output)'}
                  </pre>
                </div>
              ))}
              <div ref={logsEndRef} />
            </div>
          </div>
        </div>
      )}
    </>
  );
}
