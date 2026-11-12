import {
  batch,
  createEffect,
  createMemo,
  createSignal,
  Show,
  type Component,
} from 'solid-js';

import { getTranslatedError, isLauncherError } from '@/features/errors';
import {
  useInstallPluginFromProvider,
  usePluginPreview,
} from '@/features/plugins';
import { useTranslation } from '@/shared/i18n';
import { CombinedTextField } from '@/shared/ui-kit';
import { debounce } from '@/shared/utils';

import type { ProviderPluginPreview } from '../model/pluginProvider';
import type { ReleaseOption } from './PluginPreviewCard';

import { PluginPreviewCard } from './PluginPreviewCard';

export type InstallFromGithubViewProps = {
  onInstalled?: () => void;
};

const DEBOUNCE_MS = 500;

/** This view is the one for GitHub, so it names its provider itself. */
const PROVIDER = 'git_hub' as const;

export const InstallFromGithubView: Component<InstallFromGithubViewProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  const [url, setUrl] = createSignal('');
  const [debouncedUrl, setDebouncedUrl] = createSignal('');

  // null until the user picks a tag: the first preview is requested without one.
  const [requestedTag, setRequestedTag] = createSignal<string | null>(null);

  // Kept across tag loads so the release picker does not vanish while another tag loads.
  const [releases, setReleases] = createSignal<
    ProviderPluginPreview['releases']
  >([]);

  // Reset tag and releases together with the URL, so a new URL is never queried with the old tag.
  const applyUrl = (value: string) => {
    batch(() => {
      setRequestedTag(null);
      setReleases([]);
      setDebouncedUrl(value);
    });
  };

  const debounceUrl = debounce(applyUrl, DEBOUNCE_MS);

  const handleUrlChange = (value: string) => {
    setUrl(value);
    debounceUrl(value);
  };

  const handleUrlClear = () => {
    debounceUrl.clear();
    setUrl('');
    applyUrl('');
  };

  const previewQuery = usePluginPreview(PROVIDER, debouncedUrl, requestedTag);
  const installMutation = useInstallPluginFromProvider();

  createEffect(() => {
    const data = previewQuery.data;

    if (data) {
      setReleases(data.releases);
    }
  });

  const selectedTag = createMemo(
    () => requestedTag() ?? previewQuery.data?.tag ?? '',
  );

  const isUrlPending = () => url() !== debouncedUrl();

  // Installable only when the preview matches the selected tag and the URL shown.
  const readyPreview = createMemo(() => {
    const data = previewQuery.data;

    return data &&
      data.tag === selectedTag() &&
      !previewQuery.isFetching &&
      !isUrlPending()
      ? data
      : undefined;
  });

  const releaseOptions = createMemo<ReleaseOption[]>(() =>
    releases().map((r) => ({
      value: r.tag_name,
      label: r.is_prerelease
        ? `${r.tag_name} (${t('plugins.prerelease')})`
        : r.tag_name,
    })),
  );

  const handleReleaseChange = (option: ReleaseOption | null) => {
    if (option) {
      setRequestedTag(option.value);
    }
  };

  const handleInstall = async () => {
    const p = readyPreview();

    if (!p) {
      return;
    }

    try {
      await installMutation.mutateAsync({
        sourceType: PROVIDER,
        identifier: debouncedUrl(),
        tagName: p.tag,
        archiveSha256: p.archive_sha256,
      });
      props.onInstalled?.();
    } catch {
      // Error handled by mutation
    }
  };

  const authorsStr = createMemo(() => {
    const authors = readyPreview()?.manifest.authors;
    return authors ? authors.join(', ') : undefined;
  });

  return (
    <div class='gap-4 flex flex-col'>
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
      <Show when={previewQuery.isLoading && releases().length === 0}>
        <p class='text-sm text-muted-foreground'>{t('common.loading')}</p>
      </Show>

      {/* Error message */}
      <Show when={previewQuery.isError}>
        <p class='text-sm text-destructive'>
          {isLauncherError(previewQuery.error)
            ? getTranslatedError(previewQuery.error, t)
            : t('plugins.githubPreviewError')}
        </p>
      </Show>

      {/* Preview card */}
      <Show when={releases().length > 0 || previewQuery.data}>
        <PluginPreviewCard
          preview={readyPreview()}
          isPreviewLoading={previewQuery.isFetching || isUrlPending()}
          releaseOptions={releaseOptions()}
          selectedTag={selectedTag()}
          onReleaseChange={handleReleaseChange}
          onInstall={handleInstall}
          onClear={handleUrlClear}
          authors={authorsStr()}
          isInstalling={installMutation.isPending}
        />
      </Show>
    </div>
  );
};
