import { useState } from 'react';
import type { VmStatusUpdate } from '../lib/types.ts';

interface StatusBarProps {
  diskName: string | null;
  project: string;
  zone: string;
  vmStatus: VmStatusUpdate | undefined;
}

export default function StatusBar({ diskName, project, zone, vmStatus }: StatusBarProps) {
  const [copied, setCopied] = useState(false);
  const isRunning = vmStatus?.status === 'Running' || (!vmStatus && diskName != null);

  function buildSshCommand(): string | null {
    if (!diskName || !project || !zone) return null;
    const instanceName = vmStatus?.instanceName ?? diskName;
    const sshHost = `${instanceName}.${zone}.${project}`;
    return `ssh ${sshHost}`;
  }

  const sshCommand = buildSshCommand();

  function handleCopy() {
    if (!sshCommand) return;
    void navigator.clipboard.writeText(sshCommand).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }

  return (
    <div className="bg-[var(--color-bg-panel)] border-t border-[var(--color-border-default)] px-5 py-2 text-xs text-[var(--color-text-muted)] flex items-center gap-3 min-h-[32px]">
      {isRunning && sshCommand ? (
        <>
          <span className="text-[var(--color-text-success)]">SSH Ready</span>
          <span className="font-mono text-[#79c0ff] bg-[var(--color-bg-input)] px-2 py-0.5 rounded-sm flex-1 truncate">
            {sshCommand}
          </span>
          <button
            className="bg-transparent border border-[var(--color-border-default)] text-[var(--color-text-muted)] px-2 py-0.5 rounded cursor-pointer hover:border-[var(--color-text-link)] hover:text-[var(--color-text-link)] shrink-0"
            onClick={handleCopy}
          >
            {copied ? 'Copied!' : 'Copy'}
          </button>
        </>
      ) : (
        <span>No running VM selected</span>
      )}
    </div>
  );
}
