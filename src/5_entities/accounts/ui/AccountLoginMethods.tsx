import type { Component, ComponentProps } from 'solid-js';

import { Icon } from '@iconify-icon/solid';
import MdiCloudOffOutline from '@iconify/icons-mdi/cloud-off-outline';
import MdiSignIn from '@iconify/icons-mdi/login-variant';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button, CombinedTooltip } from '@/shared/ui';

import type { AccountType } from '../model';

export type AccountLoginMethodsProps = {
  onLogin: (type: AccountType) => void;
} & ComponentProps<'div'>;

export const AccountLoginMethods: Component<AccountLoginMethodsProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['onLogin', 'class']);

  const [{ t }] = useTranslation();

  const onOnline = () => {
    local.onLogin('online');
  };

  const onOffline = () => {
    local.onLogin('offline');
  };

  return (
    <div class={cn('flex gap-2', local.class)} {...others}>
      <CombinedTooltip
        as={Button}
        class='px-2'
        // TODO: implement minecraft login
        disabled
        label={t('account.signInMinecraft')}
        onClick={onOnline}
        variant='outline'
      >
        <span class='flex items-center gap-2'>
          {t('account.signIn')}
          <Icon class='text-2xl' icon={MdiSignIn} />
        </span>
      </CombinedTooltip>
      <CombinedTooltip
        as={Button}
        class='px-2'
        label={t('account.signInOffline')}
        onClick={onOffline}
        variant='outline'
      >
        <span class='flex items-center gap-2'>
          {t('account.offline')}
          <Icon class='text-2xl' icon={MdiCloudOffOutline} />
        </span>
      </CombinedTooltip>
    </div>
  );
};
