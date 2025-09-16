import type { ValidComponent } from 'solid-js';

import MdiDelete from '@iconify/icons-mdi/delete';
import { Polymorphic, type PolymorphicProps } from '@kobalte/core';
import { splitProps } from 'solid-js';

import type { ButtonProps } from '@/shared/ui';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton } from '@/shared/ui';

import type { AccountType } from '../model';

import { AccountButton } from './AccountButton';

export type AccountCardProps = { class?: string } & {
  username: string;
  active: boolean;
  accountType: AccountType;
  accountButtonProps?: ButtonProps;
  removeButtonProps?: ButtonProps;
  onActivate?: () => void;
  onRemove?: () => void;
};

export const AccountCard = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, AccountCardProps>,
) => {
  const [local, others] = splitProps(props, [
    'username',
    'active',
    'accountType',
    'onActivate',
    'onRemove',
    'accountButtonProps',
    'removeButtonProps',
    'class',
  ]);

  const [{ t }] = useTranslation();

  return (
    <Polymorphic
      class={cn('flex justify-between border rounded-md h-12', local.class)}
      {...others}
    >
      <AccountButton
        username={local.username}
        active={local.active}
        accountType={local.accountType}
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
    </Polymorphic>
  );
};
