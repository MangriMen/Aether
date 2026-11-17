import { createMemo, For, Show, type Component } from 'solid-js';

import {
  ErrorDetails,
  getErrorDetails,
  getTranslatedError,
  isInternalError,
  isLauncherError,
} from '@/features/errors';
import { useTranslation } from '@/shared/i18n';
import {
  Badge,
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  FieldLabel,
} from '@/shared/ui-kit';

import type { Plugin, PluginUpdateDiff } from '../model/plugin';

import { usePluginUpdatePreview, useUpdatePlugin } from '../api/pluginQueries';
import { PluginAllowedHostsSection } from './PluginAllowedHostsSection';
import { PluginHostFunctionsSection } from './PluginHostFunctionsSection';

const ChangeList: Component<{
  label: string | undefined;
  /** Shown only when it has items. */
  items: { text: string; warn?: boolean; mono?: boolean }[];
}> = (props) => (
  <Show when={props.items.length > 0}>
    <div class='gap-1.5 flex flex-col'>
      <FieldLabel>{props.label}</FieldLabel>
      <div class='gap-1.5 flex flex-wrap'>
        <For each={props.items}>
          {(item) => (
            <Badge
              variant={item.warn ? 'warning' : 'outline'}
              class={item.mono ? 'font-mono' : undefined}
            >
              {item.text}
            </Badge>
          )}
        </For>
      </div>
    </div>
  </Show>
);

/** What the new version asks for against the installed one, as the core compared it. */
const PluginUpdateDiffView: Component<{ diff: PluginUpdateDiff }> = (props) => {
  const [{ t }] = useTranslation();

  const capability = (c: PluginUpdateDiff['added_capabilities'][number]) => {
    const kind = t(`plugins.updateDialog.capabilityKind.${c.kind}`);

    return { text: `${c.name} (${kind})` };
  };

  const warnings = (): (string | undefined)[] => [
    ...props.diff.added_risks.map((risk) => t(`plugin.hostRisk.${risk}`)),
    ...(props.diff.added_allowed_hosts.some((host) => host.wide)
      ? [t('plugin.allowedHostsWide')]
      : []),
  ];

  return (
    <Show
      when={!props.diff.is_empty}
      fallback={<p class='text-sm'>{t('plugins.updateDialog.noChanges')}</p>}
    >
      <div class='gap-3 flex flex-col' data-testid='plugin-update-diff'>
        <Show when={props.diff.adds_risk}>
          <div
            role='alert'
            class='gap-1 rounded-md p-3 flex flex-col border border-warning'
          >
            <span class='text-sm font-medium text-warning'>
              {t('plugins.updateDialog.addsRisk')}
            </span>
            <For each={warnings()}>
              {(text) => <span class='text-sm text-warning'>{text}</span>}
            </For>
          </div>
        </Show>
        <ChangeList
          label={t('plugins.updateDialog.addedHostFunctions')}
          items={props.diff.added_host_functions.map((f) => ({
            text: f.name,
            warn: f.risk !== null,
            mono: true,
          }))}
        />
        <ChangeList
          label={t('plugins.updateDialog.addedHosts')}
          items={props.diff.added_allowed_hosts.map((host) => ({
            text: host.entry,
            warn: host.wide,
            mono: true,
          }))}
        />
        <ChangeList
          label={t('plugins.updateDialog.addedCapabilities')}
          items={props.diff.added_capabilities.map(capability)}
        />
        <ChangeList
          label={t('plugins.updateDialog.removedHostFunctions')}
          items={props.diff.removed_host_functions.map((f) => ({
            text: f.name,
            mono: true,
          }))}
        />
        <ChangeList
          label={t('plugins.updateDialog.removedHosts')}
          items={props.diff.removed_allowed_hosts.map((entry) => ({
            text: entry,
            mono: true,
          }))}
        />
        <ChangeList
          label={t('plugins.updateDialog.removedCapabilities')}
          items={props.diff.removed_capabilities.map(capability)}
        />
      </div>
    </Show>
  );
};

const PluginUpdateBody: Component<{
  plugin: Plugin;
  tag: string;
  onClose: () => void;
}> = (props) => {
  const [{ t }] = useTranslation();

  const pluginId = () => props.plugin.manifest.metadata.id;
  const preview = usePluginUpdatePreview(pluginId, () => props.tag);
  const updatePlugin = useUpdatePlugin();

  // Only a preview of this tag, read to the end, can be confirmed: the update is given the hash
  // of the archive it shows and installs that archive or nothing.
  const ready = createMemo(() => {
    const data = preview.data;

    return data && data.release.tag === props.tag && !preview.isFetching
      ? data
      : undefined;
  });

  const handleUpdate = async () => {
    const data = ready();

    if (!data) {
      return;
    }

    try {
      await updatePlugin.mutateAsync({
        id: pluginId(),
        tag: data.release.tag,
        archiveSha256: data.release.archive_sha256,
      });
      props.onClose();
    } catch {
      // The mutation shows the error. The release may have changed since the preview: what it
      // asks for now is shown again before another try.
      void preview.refetch();
    }
  };

  const errorMessage = (err: Error) =>
    isLauncherError(err)
      ? getTranslatedError(err, t)
      : t('plugins.updateDialog.previewError');

  return (
    <>
      <DialogHeader>
        <DialogTitle>
          {t('plugins.updateDialog.title', {
            name: props.plugin.manifest.metadata.name,
          })}
        </DialogTitle>
        <Show when={ready()}>
          {(data) => (
            <DialogDescription>
              {t('plugins.updateDialog.description', {
                from: data().installed_version,
                to: data().release.manifest.version,
              })}
            </DialogDescription>
          )}
        </Show>
      </DialogHeader>

      <Show
        when={ready()}
        fallback={
          <Show
            when={!preview.isFetching && preview.error}
            fallback={
              <p class='text-sm text-muted-foreground'>{t('common.loading')}</p>
            }
          >
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
        }
      >
        {(data) => (
          <>
            <PluginUpdateDiffView diff={data().diff} />
            <details class='text-sm'>
              <summary class='cursor-pointer text-muted-foreground'>
                {t('plugins.updateDialog.allPermissions')}
              </summary>
              <div class='mt-3 gap-4 flex flex-col'>
                <PluginAllowedHostsSection
                  allowedHosts={data().release.manifest.allowed_hosts}
                  wideHosts={data().release.manifest.wide_allowed_hosts}
                />
                <PluginHostFunctionsSection
                  hostFunctions={data().release.manifest.host_functions}
                  risks={data().release.manifest.host_function_risks}
                />
              </div>
            </details>
          </>
        )}
      </Show>

      <DialogFooter>
        <Button variant='outline' onClick={() => props.onClose()}>
          {t('common.cancel')}
        </Button>
        <Button
          variant={ready()?.diff.adds_risk ? 'warning' : 'default'}
          disabled={!ready() || updatePlugin.isPending}
          loading={updatePlugin.isPending}
          onClick={() => void handleUpdate()}
        >
          {t('plugins.updateDialog.confirm')}
        </Button>
      </DialogFooter>
    </>
  );
};

export type PluginUpdateDialogProps = {
  plugin: Plugin;
  /** The release to update to, one the update check listed; `null` keeps the dialog closed. */
  tag: string | null;
  onClose: () => void;
};

/**
 * Shows what a release of the plugin asks for against the installed version and updates to it
 * once the user confirms. The update installs exactly the archive shown, or nothing.
 */
export const PluginUpdateDialog: Component<PluginUpdateDialogProps> = (
  props,
) => (
  <Dialog
    open={props.tag !== null}
    onOpenChange={(open) => {
      if (!open) {
        props.onClose();
      }
    }}
  >
    <DialogContent>
      <Show when={props.tag} keyed>
        {(tag) => (
          <PluginUpdateBody
            plugin={props.plugin}
            tag={tag}
            onClose={props.onClose}
          />
        )}
      </Show>
    </DialogContent>
  </Dialog>
);
