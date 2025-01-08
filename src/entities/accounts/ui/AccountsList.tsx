import {
  Component,
  ComponentProps,
  Show,
  For,
  splitProps,
  createMemo,
} from 'solid-js';

import { cn } from '@/shared/lib';

import { Account } from '../model';

import { AccountCard } from './AccountCard';

export type AccountsListProps = ComponentProps<'div'> & {
  accounts: Account[];
  onActivate: (id: Account['id']) => void;
  onRemove: (id: Account['id']) => void;
};

export const AccountsList: Component<AccountsListProps> = (props) => {
  const [local, others] = splitProps(props, [
    'accounts',
    'onActivate',
    'onRemove',
    'class',
  ]);

  const mappedAccounts = createMemo(() =>
    local.accounts.reduce<{ active: Account | undefined; others: Account[] }>(
      (acc, account) => {
        if (account.active) {
          acc.active = account;
        } else {
          acc.others = acc.others || [];
          acc.others.push(account);
        }

        return acc;
      },
      { active: undefined, others: [] },
    ),
  );

  return (
    <Show
      when={local.accounts.length > 0}
      fallback={
        <span class={cn('inline-flex w-full justify-center', local.class)}>
          No accounts found
        </span>
      }
    >
      <div class={cn('flex  flex-col gap-2 ', local.class)} {...others}>
        <Show when={mappedAccounts().active}>
          {(account) => (
            <AccountCard
              username={account().username}
              active={true}
              type={account().accountType}
              onActivate={() => local.onActivate(account().id)}
              onRemove={() => local.onRemove(account().id)}
            />
          )}
        </Show>
        <For each={mappedAccounts().others}>
          {(account) => (
            <AccountCard
              username={account.username}
              active={false}
              type={account.accountType}
              onActivate={() => local.onActivate(account.id)}
              onRemove={() => local.onRemove(account.id)}
            />
          )}
        </For>
      </div>
    </Show>
  );
};
