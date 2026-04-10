import { invoke } from '@tauri-apps/api/core';

import type { AccountDto } from '@/shared/api';

const invokeAccount: typeof invoke = (cmd, ...args) =>
  invoke(`plugin:auth|${cmd}`, ...args);

export const listAccountsRaw = () =>
  invokeAccount<AccountDto[]>(`get_accounts`);

export const createOfflineAccountRaw = (username: string) =>
  invokeAccount<AccountDto>(`create_offline_account`, { username });

export const changeAccountRaw = (id: AccountDto['id']) =>
  invokeAccount<AccountDto>(`change_account`, { id });

export const logoutRaw = (id: AccountDto['id']) =>
  invokeAccount(`logout`, { id });
