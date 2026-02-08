import IconMdiCheck from '~icons/mdi/check';
import IconMdiDownload from '~icons/mdi/download';
import {
  createMemo,
  splitProps,
  type Component,
  type ComponentProps,
  Show,
} from 'solid-js';

import type { ContentItem } from '@/entities/instances';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button, Image } from '@/shared/ui';

import { useContentContext } from '../model/contentContext';

export type ContentListItemProps = ComponentProps<'div'> & {
  item: ContentItem;
  instanceId?: string;
};

export const ContentListItem: Component<ContentListItemProps> = (props) => {
  const [local, others] = splitProps(props, ['item', 'instanceId', 'class']);

  const [{ t }] = useTranslation();

  const [
    context,
    { installContent, createIsContentInstalling, createIsContentInstalled },
  ] = useContentContext();

  const contentMetadataId = createMemo(() => {
    const providerDataContentIdField = context.providerDataContentIdField;
    const providerData = local.item.providerData;

    if (!providerDataContentIdField || !providerData) {
      return;
    }

    const metadataId = providerData[providerDataContentIdField];

    if (typeof metadataId !== 'string') {
      return;
    }

    return metadataId;
  });

  const isInstalling = createIsContentInstalling(() => contentMetadataId());
  const isInstalled = createIsContentInstalled(
    () => contentMetadataId(),
    () => local.instanceId,
  );

  const installButtonText = createMemo(() => {
    if (isInstalling()) {
      return t('common.installing');
    } else if (isInstalled()) {
      return t('common.installed');
    } else {
      return t('common.install');
    }
  });

  const handleInstall = () => {
    installContent(local.item);
  };

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
            <Show when={isInstalled()} fallback={<IconMdiDownload />}>
              <IconMdiCheck />
            </Show>
          )}
          onClick={handleInstall}
          loading={isInstalling()}
          disabled={isInstalled()}
        >
          {installButtonText()}
        </Button>
      </div>
    </div>
  );
};
