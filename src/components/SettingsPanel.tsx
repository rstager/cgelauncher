import { useState, useEffect } from 'react';
import type { UserPreferences, AuthStatus } from '../lib/types.ts';
import { checkAuth, startOAuthLogin, revokeOauth } from '../lib/tauri.ts';

const ZONES = [
  'us-central1-a',
  'us-central1-b',
  'us-central1-c',
  'us-west1-a',
  'us-west1-b',
  'us-east1-b',
  'europe-west4-a',
  'europe-west4-b',
  'asia-east1-a',
];

interface SettingsPanelProps {
  preferences: UserPreferences;
  onSave: (preferences: UserPreferences) => void;
  onClose: () => void;
}

export default function SettingsPanel({ preferences, onSave, onClose }: SettingsPanelProps) {
  const [project, setProject] = useState(preferences.project);
  const [zone, setZone] = useState(preferences.zone);
  const [executionMode, setExecutionMode] = useState<'gcloud' | 'api'>(preferences.executionMode ?? 'gcloud');
  const [authStatus, setAuthStatus] = useState<AuthStatus | null>(null);
  const [authError, setAuthError] = useState<string | null>(null);
  const [oauthLoading, setOauthLoading] = useState(false);

  useEffect(() => {
    void checkAuth().then(setAuthStatus).catch(() => {});
  }, []);

  function handleSave() {
    onSave({
      ...preferences,
      project,
      zone,
      executionMode,
    });
    onClose();
  }

  async function handleOAuthLogin() {
    setOauthLoading(true);
    setAuthError(null);
    try {
      const status = await startOAuthLogin();
      setAuthStatus(status);
    } catch (err) {
      setAuthError(err instanceof Error ? err.message : String(err));
    } finally {
      setOauthLoading(false);
    }
  }

  async function handleOAuthRevoke() {
    setAuthError(null);
    try {
      await revokeOauth();
      setAuthStatus({ authenticated: false, method: '', account: null });
    } catch (err) {
      setAuthError(err instanceof Error ? err.message : String(err));
    }
  }

  return (
    <div className="settings-overlay" onClick={onClose}>
      <div className="settings-modal" onClick={(e) => e.stopPropagation()}>
        <div className="flex justify-between items-center mb-5">
          <h2 className="text-lg font-semibold text-[var(--color-text-primary)]">Settings</h2>
          <button
            className="bg-transparent border-none text-[var(--color-text-muted)] text-xl cursor-pointer hover:text-[var(--color-text-secondary)]"
            onClick={onClose}
          >
            &times;
          </button>
        </div>

        <div className="mb-4">
          <label className="block text-[13px] text-[var(--color-text-muted)] mb-1">Project ID</label>
          <input
            className="input-field"
            type="text"
            value={project}
            onChange={(e) => setProject(e.target.value)}
            placeholder="my-gcp-project"
          />
        </div>

        <div className="mb-4">
          <label className="block text-[13px] text-[var(--color-text-muted)] mb-1">Zone</label>
          <select
            className="select-field w-full"
            value={zone}
            onChange={(e) => setZone(e.target.value)}
          >
            {ZONES.map((z) => (
              <option key={z} value={z}>{z}</option>
            ))}
          </select>
        </div>

        <div className="mb-4">
          <label className="block text-[13px] text-[var(--color-text-muted)] mb-1">Authentication</label>
          <select
            className="select-field w-full"
            value={executionMode}
            onChange={(e) => setExecutionMode(e.target.value as 'gcloud' | 'api')}
          >
            <option value="gcloud">gcloud CLI</option>
            <option value="api">Google Sign-In (no gcloud required)</option>
          </select>
        </div>

        {executionMode === 'api' && (
          <div className="mb-4">
            {authStatus?.authenticated && authStatus.method === 'oauth2' ? (
              <div className="flex items-center gap-3">
                <span className="text-xs text-[var(--color-text-success)]">
                  Signed in{authStatus.account ? `: ${authStatus.account}` : ''}
                </span>
                <button
                  className="px-3 py-1 border border-[var(--color-border-default)] bg-transparent text-[var(--color-text-muted)] rounded text-xs cursor-pointer hover:border-[var(--color-accent-red)] hover:text-[var(--color-accent-red)]"
                  onClick={() => void handleOAuthRevoke()}
                >
                  Sign out
                </button>
              </div>
            ) : (
              <button
                className="btn-action btn-start text-sm px-4 py-1.5 disabled:opacity-50"
                onClick={() => void handleOAuthLogin()}
                disabled={oauthLoading}
              >
                {oauthLoading ? 'Waiting for browser...' : 'Sign in with Google'}
              </button>
            )}
            {authError && (
              <div className="text-xs text-[var(--color-accent-red)] mt-2">{authError}</div>
            )}
          </div>
        )}

        <div className="mb-5">
          <label className="block text-[13px] text-[var(--color-text-muted)] mb-1">Auth Status</label>
          {authStatus ? (
            <div className="flex items-center gap-2">
              <span
                className={`text-[11px] px-2 py-0.5 rounded-full ${
                  authStatus.authenticated
                    ? 'bg-[var(--color-auth-bg)] text-[var(--color-text-success)]'
                    : 'bg-[#6e4040] text-[var(--color-accent-red)]'
                }`}
              >
                {authStatus.authenticated ? 'Authenticated' : 'Not Authenticated'}
              </span>
              {authStatus.account && (
                <span className="text-xs text-[var(--color-text-muted)]">
                  {authStatus.method}: {authStatus.account}
                </span>
              )}
            </div>
          ) : (
            <span className="text-xs text-[var(--color-text-muted)]">Checking...</span>
          )}
        </div>

        <div className="flex justify-end gap-3">
          <button
            className="px-4 py-2 border border-[var(--color-border-default)] bg-transparent text-[var(--color-text-muted)] rounded-md text-sm cursor-pointer hover:border-[var(--color-border-active)] hover:text-[var(--color-text-link)]"
            onClick={onClose}
          >
            Cancel
          </button>
          <button className="btn-action btn-start text-sm px-6 py-2" onClick={handleSave}>
            Save
          </button>
        </div>
      </div>
    </div>
  );
}
