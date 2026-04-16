import type { AccountDto } from '@/shared/api';

import { createPluginInvoke } from '@/shared/lib';

const invokeAuth = createPluginInvoke('auth');

export const listAccountsRaw = () => invokeAuth<AccountDto[]>(`get_accounts`);

export const createOfflineAccountRaw = (username: string) =>
  invokeAuth<AccountDto>(`create_offline_account`, { username });

export const changeAccountRaw = (id: AccountDto['id']) =>
  invokeAuth<AccountDto>(`change_account`, { id });

export const logoutRaw = (id: AccountDto['id']) => invokeAuth(`logout`, { id });
