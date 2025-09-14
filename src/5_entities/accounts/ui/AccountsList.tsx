import type { Component, ComponentProps } from 'solid-js';

import { For, splitProps } from 'solid-js';
import { TransitionGroup } from 'solid-transition-group';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import type { Account } from '../model';

import { AccountCard } from './AccountCard';

export type AccountsListProps = {
  accounts: Account[];
  onActivate: (id: Account['id']) => void;
  onRemove: (id: Account['id']) => void;
} & ComponentProps<'ul'>;

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
      class={cn('flex flex-col gap-2 relative overflow-x-hidden ', local.class)}
      {...others}
    >
      <TransitionGroup name='animate-list-item'>
        <For
          each={local.accounts}
          fallback={
            <span
              class={cn('inline-flex w-full justify-center items-center h-12')}
            >
              {t('account.notFound')}
            </span>
          }
        >
          {(account) => (
            <AccountCard
              accountType={account.accountType}
              active={account.active}
              as='li'
              class='animate-list-item w-[210.625px]'
              onActivate={() => local.onActivate(account.id)}
              onRemove={() => local.onRemove(account.id)}
              username={account.username}
            />
          )}
        </For>
      </TransitionGroup>
    </ul>
  );
};
