import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component, ComponentProps } from 'solid-js';

import IconMdiAccount from '~icons/mdi/account';
import { createSignal, splitProps } from 'solid-js';

import type { Account, AccountType } from '@/features/account';
import type { CombinedTooltipProps, IconButtonProps } from '@/shared/ui-kit';

import { useChangeAccount, useLogout } from '@/features/account';
import { useTranslation } from '@/shared/i18n';
import {
  CombinedTooltip,
  IconButton,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/shared/ui-kit';

export type AccountSelectButtonProps = IconButtonProps & {
  accounts: Account[];
  accountsMenu: Component<
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
    'accountsMenu',
    'createOfflineAccountDialog',
  ]);

  const [{ t }] = useTranslation();

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
        <PopoverTrigger
          as={(props: CombinedTooltipProps) => (
            <CombinedTooltip
              label={t('common.account')}
              placement='right'
              as={IconButton}
              variant='ghost'
              size='lg'
              icon={IconMdiAccount}
              {...others}
              {...props}
            />
          )}
        />
        <PopoverContent class='p-0 w-max'>
          <local.accountsMenu
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
