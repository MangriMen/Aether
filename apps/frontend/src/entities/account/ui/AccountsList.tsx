import type { Component, ComponentProps } from 'solid-js';

import { For, splitProps } from 'solid-js';
import { TransitionGroup } from 'solid-transition-group';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import type { Account } from '../model';

import { AccountCard } from './AccountCard';

export type AccountsListProps = ComponentProps<'ul'> & {
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

  const [{ t }] = useTranslation();

  return (
    <ul
      class={cn('gap-2 relative flex flex-col overflow-x-hidden', local.class)}
      {...others}
    >
      <TransitionGroup name='animate-list-item'>
        <For
          each={local.accounts}
          fallback={
            <span
              class={cn('h-12 inline-flex w-full items-center justify-center')}
            >
              {t('account.notFound')}
            </span>
          }
        >
          {(account) => (
            <AccountCard
              class='animate-list-item'
              as='li'
              username={account.username}
              active={account.active}
              accountType={account.accountType}
              onActivate={() => local.onActivate(account.id)}
              onRemove={() => local.onRemove(account.id)}
            />
          )}
        </For>
      </TransitionGroup>
    </ul>
  );
};
