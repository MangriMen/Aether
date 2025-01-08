import MdiDelete from '@iconify/icons-mdi/delete';
// import MdiLocationEnter from '@iconify/icons-mdi/location-enter';
import { Component, ComponentProps, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { Button, ButtonProps, IconButton } from '@/shared/ui';

import { Account, AccountType } from '../model';

export type AccountCardProps = ComponentProps<'div'> & {
  username: Account['username'];
  type: AccountType;
  active?: boolean;
  accountButtonProps?: ButtonProps;
  removeButtonProps?: ButtonProps;
  onActivate?: () => void;
  onRemove?: () => void;
};

export const AccountCard: Component<AccountCardProps> = (props) => {
  const [local, others] = splitProps(props, [
    'username',
    'type',
    'active',
    'onActivate',
    'onRemove',
    'accountButtonProps',
    'removeButtonProps',
  ]);

  return (
    <div
      class={cn('flex justify-between w-full border rounded-md h-12 ')}
      {...others}
    >
      <Button
        class={cn('size-full rounded-r-none items-start flex-col px-2', {
          'bg-muted pointer-events-none': local.active,
        })}
        variant='ghost'
        title='Activate'
        onClick={local.onActivate}
        {...local.accountButtonProps}
      >
        <span class='font-bold'>{local.username}</span>
        <span class='capitalize text-muted-foreground'>{local.type}</span>
      </Button>
      <div class='flex items-start justify-start'>
        <IconButton
          class='aspect-square size-full rounded-l-none p-0 hover:bg-destructive'
          variant='ghost'
          title='Remove'
          icon={MdiDelete}
          onClick={local.onRemove}
          {...local.removeButtonProps}
        />
      </div>
    </div>
  );
};
