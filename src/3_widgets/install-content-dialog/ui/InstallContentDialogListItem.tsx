import type { Component, ComponentProps } from 'solid-js';

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

import type { ContentManager } from '../model';

export interface InstallContentDialogListItemProps {
  item: ContentItem;
  instance: Instance;
  manager: ContentManager;
  checkCompatibilityData:
    | Record<Instance['id'], ContentCompatibilityResult>
    | undefined;
  isLoadingCheckCompatibilityData?: boolean;
  onCloseDialog?: () => void;
}

export const InstallContentDialogListItem: Component<
  ComponentProps<'div'> & InstallContentDialogListItemProps
> = (props) => {
  const [local, others] = splitProps(props, [
    'manager',
    'item',
    'instance',
    'checkCompatibilityData',
    'isLoadingCheckCompatibilityData',
    'onCloseDialog',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const isInstalledMemo = createMemo(() =>
    local.manager.createIsInstalled(
      () => local.item.id,
      () => local.instance.id,
    ),
  );
  const isInstalled = () => isInstalledMemo()();

  const isInstallingMemo = createMemo(() =>
    local.manager.createIsInstalling(
      () => local.item.id,
      () => local.instance.id,
    ),
  );
  const isInstalling = () => isInstallingMemo()();

  const isCompatible = () =>
    local.checkCompatibilityData?.[local.instance.id].isCompatible;

  const isFulfilledCompatible = () =>
    local.isLoadingCheckCompatibilityData || isCompatible();

  const handleInstall = () =>
    local.manager.installContent(local.item, local.instance.id);

  return (
    <CombinedTooltip
      as='div'
      label={t('instance.notCompatibleWithContent')}
      disableTooltip={isFulfilledCompatible()}
      class={cn('flex items-center justify-between rounded py-1.5', {
        'text-muted-foreground': !isFulfilledCompatible(),
      })}
      {...others}
    >
      <A
        href={ROUTES.INSTANCE(local.instance.id)}
        onClick={local.onCloseDialog}
      >
        <div class='flex items-center gap-1 hover:underline'>
          <Image class='h-8 w-max' src={local.instance.iconPath} />
          {local.instance.name}
        </div>
      </A>

      <Show when={isFulfilledCompatible()}>
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
