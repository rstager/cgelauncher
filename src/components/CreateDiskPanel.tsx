import { useState, useEffect, useCallback } from 'react';
import type { DiskType, ImageInfo } from '../lib/types.ts';
import { listImages } from '../lib/tauri.ts';

interface CreateDiskPanelProps {
  onCreate: (name: string, sizeGb: number, diskType: DiskType, sourceImage?: string) => Promise<void>;
  onClose: () => void;
}

const DISK_TYPES: { value: DiskType; label: string }[] = [
  { value: 'pd-balanced', label: 'Balanced (pd-balanced)' },
  { value: 'pd-ssd', label: 'SSD (pd-ssd)' },
  { value: 'pd-standard', label: 'Standard HDD (pd-standard)' },
];

const IMAGE_PROJECTS = [
  { value: 'ubuntu-os-cloud', label: 'Ubuntu' },
  { value: 'debian-cloud', label: 'Debian' },
  { value: 'rocky-linux-cloud', label: 'Rocky Linux' },
  { value: 'centos-cloud', label: 'CentOS' },
];

export default function CreateDiskPanel({ onCreate, onClose }: CreateDiskPanelProps) {
  const [name, setName] = useState('');
  const [sizeGb, setSizeGb] = useState(100);
  const [diskType, setDiskType] = useState<DiskType>('pd-balanced');
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Image picker state
  const [imageProject, setImageProject] = useState('ubuntu-os-cloud');
  const [imageFilter, setImageFilter] = useState('');
  const [images, setImages] = useState<ImageInfo[]>([]);
  const [imagesLoading, setImagesLoading] = useState(false);
  const [imagesError, setImagesError] = useState<string | null>(null);
  const [selectedImage, setSelectedImage] = useState<ImageInfo | null>(null);

  // Normalise bare terms like "ubuntu" → "name:ubuntu-*"; pass through if already has a colon
  function buildFilter(raw: string): string | undefined {
    const trimmed = raw.trim();
    if (!trimmed) return undefined;
    if (trimmed.includes(':')) return trimmed;
    // Append wildcard if not already present
    return `name:${trimmed.endsWith('*') ? trimmed : `${trimmed}*`}`;
  }

  const fetchImages = useCallback(async (project: string, filter: string) => {
    setImagesLoading(true);
    setImagesError(null);
    try {
      const results = await listImages(project, buildFilter(filter));
      // Sort: newest first (lexicographic desc works for dated image names)
      const sorted = [...results].sort((a, b) => b.name.localeCompare(a.name));
      setImages(sorted);
      if (sorted.length > 0 && !selectedImage) {
        const first = sorted[0];
        setSelectedImage(first);
        if (first.diskSizeGb) setSizeGb(parseInt(first.diskSizeGb, 10));
      }
    } catch (err) {
      setImagesError(err instanceof Error ? err.message : String(err));
      setImages([]);
    } finally {
      setImagesLoading(false);
    }
  }, [selectedImage]);

  // Fetch images when project changes
  useEffect(() => {
    void fetchImages(imageProject, imageFilter);
    setSelectedImage(null);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [imageProject]);

  const handleFilterSearch = () => {
    setSelectedImage(null);
    void fetchImages(imageProject, imageFilter);
  };

  const handleImageSelect = (image: ImageInfo) => {
    setSelectedImage(image);
    if (image.diskSizeGb) {
      setSizeGb(parseInt(image.diskSizeGb, 10));
    }
  };

  async function handleCreate() {
    if (!name.trim()) {
      setError('Disk name is required.');
      return;
    }
    setCreating(true);
    setError(null);
    try {
      await onCreate(name.trim(), sizeGb, diskType, selectedImage?.selfLink);
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setCreating(false);
    }
  }

  return (
    <div className="fixed inset-0 bg-black/50 z-40 flex items-center justify-center p-6">
      <div className="bg-[var(--color-bg-panel)] border border-[var(--color-border-default)] rounded-lg w-full max-w-2xl flex flex-col overflow-hidden">
        <div className="px-4 py-3 border-b border-[var(--color-border-default)] flex items-center">
          <h2 className="text-sm font-semibold text-[var(--color-text-primary)] mr-auto">Create Disk</h2>
          <button
            className="bg-transparent border-none text-[var(--color-text-muted)] cursor-pointer text-base px-1.5 py-0.5 rounded hover:text-[var(--color-text-secondary)]"
            onClick={onClose}
          >
            ✕
          </button>
        </div>

        <div className="p-4 space-y-4 overflow-y-auto">
          {/* Image template picker */}
          <div className="flex flex-col gap-2">
            <label className="text-xs font-medium text-[var(--color-text-muted)]">Image Template</label>
            <div className="flex gap-2">
              <select
                className="select-field flex-1"
                value={imageProject}
                onChange={(e) => setImageProject(e.target.value)}
                disabled={creating}
              >
                {IMAGE_PROJECTS.map((p) => (
                  <option key={p.value} value={p.value}>{p.label}</option>
                ))}
              </select>
            </div>
            <div className="flex gap-2">
              <input
                type="text"
                className="bg-[var(--color-bg-input)] border border-[var(--color-border-default)] text-[var(--color-text-secondary)] px-2 py-1.5 rounded text-[13px] flex-1"
                placeholder="Filter (e.g. ubuntu-2204 or name:ubuntu-*)"
                value={imageFilter}
                onChange={(e) => setImageFilter(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleFilterSearch()}
                disabled={creating}
              />
              <button
                className="bg-transparent border border-[var(--color-border-default)] text-[var(--color-text-muted)] px-3 py-1.5 rounded text-[13px] cursor-pointer hover:border-[var(--color-text-link)] hover:text-[var(--color-text-link)]"
                onClick={handleFilterSearch}
                disabled={creating || imagesLoading}
              >
                Search
              </button>
            </div>
            {imagesLoading && (
              <div className="text-xs text-[var(--color-text-muted)]">Loading images...</div>
            )}
            {imagesError && (
              <div className="text-xs text-[var(--color-accent-red)]">{imagesError}</div>
            )}
            {!imagesLoading && !imagesError && images.length > 0 && (
              <div className="overflow-x-auto">
                <select
                  className="select-field min-w-full"
                  style={{ minWidth: 'max-content' }}
                  value={selectedImage?.selfLink ?? ''}
                  onChange={(e) => {
                    const img = images.find((i) => i.selfLink === e.target.value);
                    if (img) handleImageSelect(img);
                  }}
                  disabled={creating}
                  size={Math.min(images.length, 6)}
                >
                  {images.map((img) => (
                    <option key={img.selfLink} value={img.selfLink}>
                      {img.name}{img.description ? ` — ${img.description}` : ''}
                    </option>
                  ))}
                </select>
              </div>
            )}
            {!imagesLoading && !imagesError && images.length === 0 && (
              <div className="text-xs text-[var(--color-text-muted)]">No images found.</div>
            )}
          </div>

          {/* Name */}
          <div className="flex flex-col gap-1">
            <label className="text-xs text-[var(--color-text-muted)]">Name</label>
            <input
              type="text"
              className="bg-[var(--color-bg-input)] border border-[var(--color-border-default)] text-[var(--color-text-secondary)] px-2 py-1.5 rounded text-[13px]"
              placeholder="my-disk"
              value={name}
              onChange={(e) => setName(e.target.value)}
              disabled={creating}
            />
          </div>

          {/* Size */}
          <div className="flex flex-col gap-1">
            <label className="text-xs text-[var(--color-text-muted)]">Size (GB)</label>
            <input
              type="number"
              min={10}
              max={65536}
              className="bg-[var(--color-bg-input)] border border-[var(--color-border-default)] text-[var(--color-text-secondary)] px-2 py-1.5 rounded text-[13px]"
              value={sizeGb}
              onChange={(e) => setSizeGb(Number(e.target.value))}
              disabled={creating}
            />
          </div>

          {/* Disk type */}
          <div className="flex flex-col gap-1">
            <label className="text-xs text-[var(--color-text-muted)]">Type</label>
            <select
              className="select-field"
              value={diskType}
              onChange={(e) => setDiskType(e.target.value as DiskType)}
              disabled={creating}
            >
              {DISK_TYPES.map((t) => (
                <option key={t.value} value={t.value}>{t.label}</option>
              ))}
            </select>
          </div>

          {error && (
            <div className="text-xs text-[var(--color-accent-red)]">{error}</div>
          )}
        </div>

        <div className="px-4 py-3 border-t border-[var(--color-border-default)] flex justify-end gap-2">
          <button
            className="bg-transparent border border-[var(--color-border-default)] text-[var(--color-text-muted)] px-3 py-1.5 rounded text-[13px] cursor-pointer hover:border-[var(--color-text-link)] hover:text-[var(--color-text-link)]"
            onClick={onClose}
            disabled={creating}
          >
            Cancel
          </button>
          <button
            className="btn-action btn-start text-[13px] px-4 py-1.5"
            onClick={() => void handleCreate()}
            disabled={creating || !name.trim()}
          >
            {creating ? 'Creating...' : 'Create'}
          </button>
        </div>
      </div>
    </div>
  );
}
