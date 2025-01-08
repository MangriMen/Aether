import { invoke } from '@tauri-apps/api/core';

import { Account } from '../model';

export const getAccounts = () => invoke<Account[]>('get_accounts');

export const createOfflineAccount = (username: string) =>
  invoke('create_offline_account', { username });

export const changeAccount = (id: Account['id']) =>
  invoke('change_account', { id });

export const logout = (id: Account['id']) => invoke('logout', { id });
