interface StatusBarProps {
  diskName: string | null;
  project: string;
  zone: string;
  isRunning: boolean;
}

export default function StatusBar({ diskName, project, zone, isRunning }: StatusBarProps) {
  const vmName = diskName;
  const sshHost = vmName && project && zone ? `${vmName}.${zone}.${project}` : null;

  return (
    <div className="bg-[var(--color-bg-panel)] border-t border-[var(--color-border-default)] px-5 py-2 text-xs text-[var(--color-text-muted)] flex items-center gap-3">
      {isRunning && sshHost ? (
        <>
          <span className="text-[var(--color-text-success)]">SSH Ready</span>
          <span>Connect via:</span>
          <span className="font-mono text-[#79c0ff] bg-[var(--color-bg-input)] px-2 py-0.5 rounded-sm">
            ssh {sshHost}
          </span>
        </>
      ) : (
        <span>No running VM selected</span>
      )}
    </div>
  );
}
