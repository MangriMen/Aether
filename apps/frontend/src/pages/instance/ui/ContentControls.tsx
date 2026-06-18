import {
  createMemo,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import type { Instance } from '@/entities/instances';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTextField } from '@/shared/ui';

import type { InstallContentButtonProps } from './InstallContentButton';

import { InstallContentButton } from './InstallContentButton';

export type ContentControlsProps = ComponentProps<'div'> &
  Pick<InstallContentButtonProps, 'contentTypes'> & {
    instanceId: Instance['id'];
    contentsCount: number;
    isInstalling?: boolean;
    onSearch?: (query: string) => void;
    onInstallContentClick?: () => void;
  };

export const ContentControls: Component<ContentControlsProps> = (props) => {
  const [local, others] = splitProps(props, [
    'instanceId',
    'contentsCount',
    'isInstalling',
    'onSearch',
    'onInstallContentClick',
    'contentTypes',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const searchPlaceholder = createMemo(() =>
    t('content.searchCount', { count: local.contentsCount }),
  );

  return (
    <div class={cn('flex items-center gap-2', local.class)} {...others}>
      <CombinedTextField
        class='w-full'
        name='search'
        inputProps={{ type: 'text', placeholder: searchPlaceholder() }}
        onChange={local.onSearch}
      />
      <InstallContentButton
        instanceId={local.instanceId}
        onInstallContentClick={local.onInstallContentClick}
        contentTypes={local.contentTypes}
        disabled={local.isInstalling}
      />
    </div>
  );
};
