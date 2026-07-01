import type { Component, ComponentProps } from 'solid-js';

import IconMdiCloudOffOutline from '~icons/mdi/cloud-off-outline';
import IconMdiLoginVariant from '~icons/mdi/login-variant';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button, CombinedTooltip } from '@/shared/ui';

import type { AccountType } from '../model';

export type AccountLoginMethodsProps = ComponentProps<'div'> & {
  onLogin: (type: AccountType) => void;
};

export const AccountLoginMethods: Component<AccountLoginMethodsProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['onLogin', 'class']);

  const [{ t }] = useTranslation();

  const onMicrosoft = () => {
    local.onLogin('microsoft');
  };

  const onOffline = () => {
    local.onLogin('offline');
  };

  return (
    <div class={cn('gap-2 flex whitespace-nowrap', local.class)} {...others}>
      <CombinedTooltip
        label={t('account.signInMinecraft')}
        as={Button}
        variant='outline'
        class='px-2 w-full'
        onClick={onMicrosoft}
        // TODO: implement minecraft login
        disabled
      >
        <span class='gap-2 flex items-center'>
          {t('account.signIn')}
          <IconMdiLoginVariant />
        </span>
      </CombinedTooltip>
      <CombinedTooltip
        label={t('account.signInOffline')}
        as={Button}
        variant='outline'
        class='px-2 w-full'
        onClick={onOffline}
      >
        <span class='gap-2 flex items-center'>
          {t('account.offline')}
          <IconMdiCloudOffOutline />
        </span>
      </CombinedTooltip>
    </div>
  );
};
