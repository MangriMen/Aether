import MdiAccount from '@iconify/icons-mdi/account';
import type { Component, ComponentProps } from 'solid-js';
import { createSignal, splitProps } from 'solid-js';

import type { IconButtonProps } from '@/shared/ui';
import {
  CombinedTooltip,
  IconButton,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/shared/ui';

import type { Account, AccountType } from '@/entities/accounts';
import { useChangeAccount, useLogout } from '@/entities/accounts';

import { useTranslate } from '@/shared/model';
import type { DialogRootProps } from '@kobalte/core/dialog';

export type AccountSelectButtonProps = IconButtonProps & {
  accounts: Account[];
  accountSelectCard: Component<
    ComponentProps<'div'> & {
      accounts: Account[];
      onActivate: (id: Account['id']) => void;
      onCreate: (type: AccountType) => void;
      onLogout: (uuid: string) => void;
    }
  >;
  createOfflineAccountDialog: Component<
    ComponentProps<'div'> & DialogRootProps
  >;
};

export const AccountSelectButton: Component<AccountSelectButtonProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'accounts',
    'accountSelectCard',
    'createOfflineAccountDialog',
  ]);

  const [{ t }] = useTranslate();

  const handleCreate = (type: AccountType) => {
    setAccountCreationType(type);
  };

  const { mutateAsync: changeAccount } = useChangeAccount();

  const handleSelect = async (id: Account['id']) => {
    await changeAccount(id);
  };

  const { mutateAsync: logout } = useLogout();

  const handleLogout = async (uuid: string) => {
    await logout(uuid);
  };

  const [accountCreationType, setAccountCreationType] =
    createSignal<AccountType | null>(null);

  const closeAccountCreationDialog = () => {
    setAccountCreationType(null);
  };

  return (
    <>
      <Popover>
        <PopoverTrigger>
          <CombinedTooltip
            label={t('common.account')}
            placement='right'
            as={IconButton}
            variant='ghost'
            icon={MdiAccount}
            {...others}
          />
        </PopoverTrigger>
        <PopoverContent class='w-max p-0'>
          <local.accountSelectCard
            accounts={local.accounts}
            onCreate={handleCreate}
            onActivate={handleSelect}
            onLogout={handleLogout}
          />
        </PopoverContent>
      </Popover>

      <local.createOfflineAccountDialog
        open={accountCreationType() === 'offline'}
        onOpenChange={closeAccountCreationDialog}
      />
    </>
  );
};
