import { invoke } from '@tauri-apps/api/core';

import type { Account } from '../model';

const invokeAccount: typeof invoke = (cmd, ...args) =>
  invoke(`plugin:auth|${cmd}`, ...args);

export const listAccountsRaw = () => invokeAccount<Account[]>(`get_accounts`);

export const createOfflineAccountRaw = (username: string) =>
  invokeAccount<Account>(`create_offline_account`, { username });

export const changeAccountRaw = (id: Account['id']) =>
  invokeAccount<Account>(`change_account`, { id });

export const logoutRaw = (id: Account['id']) => invokeAccount(`logout`, { id });
