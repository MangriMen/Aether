import type { Accessor, Component, ComponentProps } from 'solid-js';

import { A } from '@solidjs/router';
import { createMemo, Show, splitProps } from 'solid-js';

import type { ContentCompatibilityResult } from '@/entities/instances/model/compatibility';

import {
  ContentInstallButton,
  type ContentItem,
  type Instance,
} from '@/entities/instances';
import { ROUTES } from '@/shared/config';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTooltip, Image } from '@/shared/ui';

export interface InstallContentDialogListItemProps {
  item: ContentItem;
  itemId: string | undefined;
  instance: Instance;
  checkCompatibilityData:
    | Record<Instance['id'], ContentCompatibilityResult>
    | undefined;
  isLoadingCheckCompatibilityData?: boolean;
  installContent: (
    item: ContentItem,
    instanceId?: Instance['id'],
  ) => Promise<void>;
  createIsContentInstalling: (
    contentId: Accessor<string | undefined>,
    instanceId: Accessor<string | undefined>,
  ) => Accessor<boolean>;
  createIsContentInstalled: (
    contentId: Accessor<string | undefined>,
    instanceId: Accessor<string | undefined>,
  ) => Accessor<boolean>;
}

export const InstallContentDialogListItem: Component<
  ComponentProps<'div'> & InstallContentDialogListItemProps
> = (props) => {
  const [local, others] = splitProps(props, [
    'itemId',
    'item',
    'instance',
    'installContent',
    'createIsContentInstalled',
    'createIsContentInstalling',
    'checkCompatibilityData',
    'isLoadingCheckCompatibilityData',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const isInstalled = createMemo(() =>
    local.createIsContentInstalled(
      () => local.itemId,
      () => local.instance.id,
    )(),
  );

  const isInstalling = createMemo(() =>
    local.createIsContentInstalling(
      () => local.itemId,
      () => local.instance.id,
    )(),
  );

  const handleInstall = () =>
    local.installContent(local.item, local.instance.id);

  const isCompatible = () =>
    local.checkCompatibilityData?.[local.instance.id].isCompatible;

  return (
    <CombinedTooltip
      as='div'
      label={t('instance.notCompatibleWithContent')}
      disableTooltip={local.isLoadingCheckCompatibilityData || isCompatible()}
      class={cn('flex items-center justify-between rounded py-1.5', {
        'text-muted-foreground':
          !local.isLoadingCheckCompatibilityData && !isCompatible(),
      })}
      {...others}
    >
      <A href={ROUTES.INSTANCE(local.instance.id)}>
        <div class='flex items-center gap-1 hover:underline'>
          <Image class='h-8 w-max' src={local.instance.iconPath} />
          {local.instance.name}
        </div>
      </A>

      <Show when={isCompatible()}>
        <ContentInstallButton
          isInstalled={isInstalled()}
          isInstalling={isInstalling()}
          isLoading={local.isLoadingCheckCompatibilityData}
          onClick={handleInstall}
          size='sm'
        />
      </Show>
    </CombinedTooltip>
  );
};
