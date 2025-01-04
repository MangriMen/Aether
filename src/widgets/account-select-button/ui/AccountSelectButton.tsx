import MdiAccount from '@iconify/icons-mdi/account';
import { Icon } from '@iconify-icon/solid';
import { Component, createSignal } from 'solid-js';

import {
  IconButton,
  IconButtonProps,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/shared/ui';

import {
  Account,
  AccountType,
  changeAccount,
  logout,
  refetchAccountStateResource,
} from '@/entities/accounts';

import { AccountSelectForm } from '@/features/account-select-form';

// eslint-disable-next-line boundaries/element-types
import { CreateOfflineAccountDialog } from '@/widgets/create-offline-account-dialog';

export type AccountSelectProps = IconButtonProps;

export const AccountSelectButton: Component<AccountSelectProps> = (props) => {
  const handleSelect = async (id: Account['id']) => {
    await changeAccount(id);
    refetchAccountStateResource();
  };

  const handleCreate = (type: AccountType) => {
    setAccountCreationType(type);
  };

  const handleLogout = async (uuid: string) => {
    await logout(uuid);
    refetchAccountStateResource();
  };

  const [accountCreationType, setAccountCreationType] =
    createSignal<AccountType | null>(null);

  const closeAccountCreationDialog = () => {
    setAccountCreationType(null);
  };

  return (
    <>
      <Popover>
        <PopoverTrigger as={IconButton} variant='ghost' {...props}>
          <Icon icon={MdiAccount} class='text-2xl' />
        </PopoverTrigger>
        <PopoverContent class='w-max'>
          <AccountSelectForm
            onCreateAccount={handleCreate}
            onSelectAccount={handleSelect}
            onLogout={handleLogout}
          />
        </PopoverContent>
      </Popover>

      <CreateOfflineAccountDialog
        open={accountCreationType() === 'offline'}
        onOpenChange={closeAccountCreationDialog}
      />
    </>
  );
};
