import IconMdiCheck from '~icons/mdi/check';
import IconMdiDownload from '~icons/mdi/download';
import {
  createMemo,
  createSignal,
  splitProps,
  type Component,
  type ComponentProps,
  Show,
} from 'solid-js';

import {
  useInstallContent,
  type ContentItemExtended,
  type InstallContentPayload,
} from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button, Image } from '@/shared/ui';

export type ContentListItemProps = ComponentProps<'div'> & {
  item: ContentItemExtended;
  instanceId?: string;
  gameVersion?: string;
  loader?: string;
  provider?: string;
  onInstalled?: (providerData: ContentItemExtended['providerData']) => void;
};

export const ContentListItem: Component<ContentListItemProps> = (props) => {
  const [local, others] = splitProps(props, [
    'item',
    'instanceId',
    'gameVersion',
    'loader',
    'provider',
    'onInstalled',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const [isInstalling, setIsInstalling] = createSignal(false);

  const { mutateAsync: installContent } = useInstallContent();

  const handleInstallContent = async () => {
    if (!local.provider || !local.instanceId || !local.gameVersion) {
      return;
    }

    const payload: InstallContentPayload = {
      gameVersion: local.gameVersion,
      loader: local.loader,
      contentType: local.item.contentType,
      contentVersion: undefined,
      provider: local.provider,
      providerData: local.item.providerData,
    };

    if (local.item.contentType !== 'mod') {
      payload.loader = undefined;
    }

    setIsInstalling(true);

    try {
      await installContent({ id: local.instanceId, payload });
      local.onInstalled?.(local.item.providerData);
    } catch {
      /* empty */
    }

    setIsInstalling(false);
  };

  const installButtonText = createMemo(() => {
    if (isInstalling()) {
      return t('common.installing');
    } else if (local.item.installed) {
      return t('common.installed');
    } else {
      return t('common.install');
    }
  });

  return (
    <div
      class={cn(
        'flex gap-2 border border-secondary rounded-lg p-3',
        local.class,
      )}
      {...others}
    >
      <Image
        class='aspect-square size-24'
        src={local.item.iconUrl || undefined}
      />
      <div class='flex flex-col text-muted-foreground'>
        <span class='text-lg font-bold text-foreground'>
          <a href={local.item.url} target='_blank'>
            {local.item.name}
          </a>{' '}
          <span class='text-base font-semibold text-muted-foreground'>
            by {local.item.author}
          </span>
        </span>
        <span>{local.item.description}</span>
      </div>
      <div class='ml-auto flex flex-col justify-end'>
        <Button
          class='px-3'
          leadingIcon={() => (
            <Show when={local.item.installed} fallback={<IconMdiDownload />}>
              <IconMdiCheck />
            </Show>
          )}
          onClick={handleInstallContent}
          loading={isInstalling()}
          disabled={local.item.installed}
        >
          {installButtonText()}
        </Button>
      </div>
    </div>
  );
};
