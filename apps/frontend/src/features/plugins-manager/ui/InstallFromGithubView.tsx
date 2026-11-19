import {
  batch,
  createEffect,
  createMemo,
  createSignal,
  Show,
  type Component,
} from 'solid-js';

import {
  ErrorDetails,
  getErrorDetails,
  getTranslatedError,
  isInternalError,
  isLauncherError,
} from '@/features/errors';
import {
  useInstallPluginFromProvider,
  usePluginPreview,
  usePluginReleases,
} from '@/features/plugins';
import { useTranslation } from '@/shared/i18n';
import { CombinedTextField } from '@/shared/ui-kit';
import { debounce } from '@/shared/utils';

import type { ReleaseOption } from './PluginPreviewCard';

import { PluginPreviewCard } from './PluginPreviewCard';

export type InstallFromGithubViewProps = {
  onInstalled?: () => void;
};

const DEBOUNCE_MS = 500;

/** This view is the one for GitHub, so it names its provider itself. */
const PROVIDER = 'github' as const;

export const InstallFromGithubView: Component<InstallFromGithubViewProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  const [url, setUrl] = createSignal('');
  const [debouncedUrl, setDebouncedUrl] = createSignal('');

  // null until the user picks a tag: the listing says which one to start from.
  const [pickedTag, setPickedTag] = createSignal<string | null>(null);

  // Reset the picked tag together with the URL, so a new URL is never previewed with the old tag.
  const applyUrl = (value: string) => {
    batch(() => {
      setPickedTag(null);
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

  const releasesQuery = usePluginReleases(PROVIDER, debouncedUrl);

  // The list of the URL shown: while the query still holds the list of the previous URL, there
  // is none, so a new URL is never previewed with a tag of the old one.
  const listing = () => {
    const data = releasesQuery.data;

    return data?.url === debouncedUrl() ? data : undefined;
  };

  // The tag to preview is pinned once the list arrives, so a refreshed list does not change the
  // release the user is looking at.
  createEffect(() => {
    const data = listing();

    if (data && pickedTag() === null) {
      setPickedTag(data.default_tag);
    }
  });

  // A pinned tag the list no longer has falls back to the default one.
  const selectedTag = createMemo(() => {
    const data = listing();
    const picked = pickedTag();

    if (!data) {
      return '';
    }

    return picked && data.releases.some((r) => r.tag_name === picked)
      ? picked
      : data.default_tag;
  });

  const previewQuery = usePluginPreview(PROVIDER, debouncedUrl, selectedTag);
  const installMutation = useInstallPluginFromProvider();

  const errorMessage = (err: Error) =>
    isLauncherError(err)
      ? getTranslatedError(err, t)
      : t('plugins.githubPreviewError');

  const failure = () => releasesQuery.error ?? previewQuery.error;

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
    (listing()?.releases ?? []).map((r) => ({
      value: r.tag_name,
      label: r.is_prerelease
        ? `${r.tag_name} (${t('plugins.prerelease')})`
        : r.tag_name,
    })),
  );

  const handleReleaseChange = (option: ReleaseOption | null) => {
    if (option) {
      setPickedTag(option.value);
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
      <Show when={releasesQuery.isLoading}>
        <p class='text-sm text-muted-foreground'>{t('common.loading')}</p>
      </Show>

      {/* Error message: of the list, else of the preview */}
      <Show when={failure()}>
        {(err) => (
          <>
            <p class='text-sm text-destructive'>{errorMessage(err())}</p>
            <Show when={getErrorDetails(err())}>
              {(details) => (
                <ErrorDetails
                  details={details()}
                  defaultOpen={isInternalError(err())}
                />
              )}
            </Show>
          </>
        )}
      </Show>

      {/* Preview card */}
      <Show when={listing()}>
        {(data) => (
          <PluginPreviewCard
            repo={data().repo}
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
        )}
      </Show>
    </div>
  );
};
