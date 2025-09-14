import {
  type Component,
  type ComponentProps,
  createMemo,
  splitProps,
} from 'solid-js';

import type { Instance } from '@/entities/instances';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTextField } from '@/shared/ui';

import type { InstallContentButtonProps } from './InstallContentButton';

import { InstallContentButton } from './InstallContentButton';

export type ContentControlsProps = {
  contentsCount: number;
  instanceId: Instance['id'];
  isInstalling?: boolean;
  onInstallContentClick?: () => void;
  onSearch?: (query: string) => void;
} & ComponentProps<'div'> &
  Pick<InstallContentButtonProps, 'contentTypes'>;

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
        inputProps={{ placeholder: searchPlaceholder(), type: 'text' }}
        name='search'
        onChange={local.onSearch}
      />
      <InstallContentButton
        contentTypes={local.contentTypes}
        disabled={local.isInstalling}
        instanceId={local.instanceId}
        onInstallContentClick={local.onInstallContentClick}
      />
    </div>
  );
};
