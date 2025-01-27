import MdiDelete from '@iconify/icons-mdi/delete';
import type { Component, ComponentProps } from 'solid-js';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import type { ButtonProps } from '@/shared/ui';
import { CombinedTooltip, IconButton } from '@/shared/ui';

// eslint-disable-next-line boundaries/element-types
import { useTranslate } from '@/app/model';

import type { Account, AccountType } from '../model';

import AccountButton from './AccountButton';

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

  const [{ t }] = useTranslate();

  return (
    <div
      class={cn('flex justify-between w-full border rounded-md h-12 ')}
      {...others}
    >
      <AccountButton
        active={local.active}
        username={local.username}
        type={local.type}
        onClick={local.onActivate}
        {...local.accountButtonProps}
      />
      <div class='flex items-start justify-start'>
        <CombinedTooltip
          label={t('account.removeAccount')}
          as={IconButton}
          class='aspect-square size-full rounded-l-none p-0 hover:bg-destructive focus:z-10'
          variant='ghost'
          icon={MdiDelete}
          onClick={local.onRemove}
          {...local.removeButtonProps}
        />
      </div>
    </div>
  );
};
