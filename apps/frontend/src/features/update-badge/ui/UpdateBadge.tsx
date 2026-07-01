import {
  createMemo,
  type Component,
  Show,
  splitProps,
  mergeProps,
} from 'solid-js';

import type { BadgeProps } from '@/shared/ui';

import { cn } from '@/shared/lib';
import { Badge } from '@/shared/ui';

import {
  checkIsUpdateAvailable,
  useCheckUpdate,
} from '../../../entities/updates/model';

export type UpdateBadgeProps = BadgeProps & {
  isAbleToShow?: boolean;
};

export const UpdateBadge: Component<UpdateBadgeProps> = (props) => {
  const merged = mergeProps({ isAbleToShow: true }, props);

  const [local, others] = splitProps(merged, ['isAbleToShow', 'class']);

  const update = useCheckUpdate();

  const isUpdateAvailable = createMemo(() =>
    update.data ? checkIsUpdateAvailable(update.data) : false,
  );

  return (
    <Show when={local.isAbleToShow && isUpdateAvailable()}>
      <Badge class={cn('size-2 p-0 aspect-square', local.class)} {...others} />
    </Show>
  );
};
