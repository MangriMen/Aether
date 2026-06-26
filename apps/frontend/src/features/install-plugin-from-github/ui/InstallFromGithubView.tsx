import {
  createEffect,
  createMemo,
  createSignal,
  Show,
  type Component,
} from 'solid-js';

import type { ProviderPluginPreviewDto } from '@/entities/plugins';

import {
  useInstallPluginFromProvider,
  usePluginPreview,
} from '@/entities/plugins';
import { debounce } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTextField } from '@/shared/ui';

import type { ReleaseOption } from './PluginPreviewCard';

import { PluginPreviewCard } from './PluginPreviewCard';

export type InstallFromGithubViewProps = {
  onInstalled?: () => void;
};

const DEBOUNCE_MS = 500;

export const InstallFromGithubView: Component<InstallFromGithubViewProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  const [url, setUrl] = createSignal('');
  const [debouncedUrl, setDebouncedUrl] = createSignal('');

  const debounceUrl = debounce(
    (value: string) => setDebouncedUrl(value),
    DEBOUNCE_MS,
  );

  const handleUrlChange = (value: string) => {
    setUrl(value);
    debounceUrl(value);
  };

  const handleUrlClear = () => {
    setUrl('');
    setDebouncedUrl('');
  };

  const previewQuery = usePluginPreview(debouncedUrl);
  const installMutation = useInstallPluginFromProvider();

  const previewData = createMemo<ProviderPluginPreviewDto | null | undefined>(
    () => previewQuery.data,
  );

  const [selectedTag, setSelectedTag] = createSignal('');

  // Auto-select first release when preview loads
  createEffect(() => {
    const data = previewData();
    const firstRelease = data?.releases?.[0];
    if (firstRelease && !selectedTag()) {
      setSelectedTag(firstRelease.tag_name);
    }
  });

  const releaseOptions = createMemo<ReleaseOption[]>(() => {
    const p = previewData();
    if (!p) return [];
    return p.releases.map((r) => ({
      value: r.tag_name,
      label: r.is_prerelease
        ? `${r.tag_name} (${t('plugins.prerelease')})`
        : r.tag_name,
    }));
  });

  const handleReleaseChange = (option: ReleaseOption | null) => {
    if (option) setSelectedTag(option.value);
  };

  const handleInstall = async () => {
    const p = previewData();
    const tag = selectedTag();
    if (!p || !tag) return;

    const release = p.releases.find((r) => r.tag_name === tag);
    if (!release) return;

    try {
      await installMutation.mutateAsync({
        identifier: debouncedUrl(),
        downloadUrl: release.download_url,
        tagName: release.tag_name,
        version: release.version,
      });
      props.onInstalled?.();
    } catch {
      // Error handled by mutation
    }
  };

  const authorsStr = createMemo(() => {
    const authors = previewData()?.manifest?.authors;
    return authors ? authors.join(', ') : undefined;
  });

  return (
    <div class='flex flex-col gap-4'>
      {/* URL input row */}
      <CombinedTextField
        class='flex-1'
        label={t('plugins.githubUrl')}
        inputProps={{
          placeholder: 'https://github.com/owner/repo',
        }}
        value={url()}
        onChange={handleUrlChange}
      />

      {/* Loading indicator */}
      <Show when={previewQuery.isLoading}>
        <p class='text-sm text-muted-foreground'>{t('common.loading')}</p>
      </Show>

      {/* Error message */}
      <Show when={previewQuery.isError}>
        <p class='text-sm text-destructive'>
          {previewQuery.error instanceof Error
            ? previewQuery.error.message
            : t('plugins.githubPreviewError')}
        </p>
      </Show>

      {/* Preview card */}
      <Show when={previewData()}>
        {(p) => (
          <PluginPreviewCard
            preview={p()}
            releaseOptions={releaseOptions()}
            selectedTag={selectedTag()}
            onReleaseChange={handleReleaseChange}
            onInstall={handleInstall}
            onClear={handleUrlClear}
            authors={authorsStr()}
            isInstalling={installMutation.isPending}
          />
        )}
      </Show>
    </div>
  );
};
