import type { Component, ComponentProps } from 'solid-js';

import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { Separator } from '@/shared/ui';

import type { Account, AccountType } from '../model';

import { useSortedAccounts } from '../lib';
import { AccountLoginMethods } from './AccountLoginMethods';
import { AccountsList } from './AccountsList';

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
    'class',
  ]);

  const sortedAccounts = useSortedAccounts(() => local.accounts);

  return (
    <div class={cn('w-56', local.class)} {...others}>
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
