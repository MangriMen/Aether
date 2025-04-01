import {
  createMemo,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { CombinedTextField } from '@/shared/ui';
import { cn } from '@/shared/lib';
import type { InstallContentButtonProps } from './InstallContentButton';
import { InstallContentButton } from './InstallContentButton';
import type { Instance } from '@/entities/instances';

export type ContentControlsProps = ComponentProps<'div'> &
  Pick<InstallContentButtonProps, 'contentTypes'> & {
    instanceId: Instance['id'];
    contentsCount: number;
    onSearch?: (query: string) => void;
    onInstallContentClick?: () => void;
  };

export const ContentControls: Component<ContentControlsProps> = (props) => {
  const [local, others] = splitProps(props, [
    'instanceId',
    'contentsCount',
    'onSearch',
    'onInstallContentClick',
    'contentTypes',
    'class',
  ]);

  const searchPlaceholder = createMemo(
    () => `Search ${local.contentsCount} contents`,
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
      />
    </div>
  );
};
