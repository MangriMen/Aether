import type { Component, ComponentProps } from 'solid-js';

import { splitProps } from 'solid-js';

import type { Account, AccountType } from '@/entities/accounts';

import { AccountLoginMethods, AccountsList } from '@/entities/accounts';
import { Separator } from '@/shared/ui';

import { useSortedAccounts } from '../lib';

export type AccountsMenuProps = ComponentProps<'div'> & {
  accounts: Account[];
  onActivate: (id: Account['id']) => void;
  onCreate: (type: AccountType) => void;
  onLogout: (id: Account['id']) => void;
};

export const AccountsMenu: Component<AccountsMenuProps> = (props) => {
  const [local, others] = splitProps(props, [
    'accounts',
    'onActivate',
    'onCreate',
    'onLogout',
  ]);

  const sortedAccounts = useSortedAccounts(() => local.accounts);

  return (
    <div {...others}>
      <AccountsList
        class='max-h-48 overflow-y-auto p-3'
        accounts={sortedAccounts()}
        onActivate={local.onActivate}
        onRemove={local.onLogout}
      />
      <Separator class='mb-3' />
      <AccountLoginMethods class='px-3 pb-3' onLogin={local.onCreate} />
    </div>
  );
};
