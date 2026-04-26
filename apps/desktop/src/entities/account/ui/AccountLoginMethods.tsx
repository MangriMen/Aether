import type { Component, ComponentProps } from 'solid-js';

import IconMdiCloudOffOutline from '~icons/mdi/cloud-off-outline';
import IconMdiLoginVariant from '~icons/mdi/login-variant';
import { splitProps } from 'solid-js';

import type { AccountType } from '../model';

import { cn } from '../../../shared/lib';
import { useTranslation } from '../../../shared/model';
import { Button, CombinedTooltip } from '../../../shared/ui';

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
    <div class={cn('flex gap-2 whitespace-nowrap', local.class)} {...others}>
      <CombinedTooltip
        label={t('account.signInMinecraft')}
        as={Button}
        variant='outline'
        class='w-full px-2'
        onClick={onMicrosoft}
        // TODO: implement minecraft login
        disabled
      >
        <span class='flex items-center gap-2'>
          {t('account.signIn')}
          <IconMdiLoginVariant />
        </span>
      </CombinedTooltip>
      <CombinedTooltip
        label={t('account.signInOffline')}
        as={Button}
        variant='outline'
        class='w-full px-2'
        onClick={onOffline}
      >
        <span class='flex items-center gap-2'>
          {t('account.offline')}
          <IconMdiCloudOffOutline />
        </span>
      </CombinedTooltip>
    </div>
  );
};
