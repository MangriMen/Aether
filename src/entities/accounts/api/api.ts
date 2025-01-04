import { invoke } from '@tauri-apps/api/core';

import { Account, AccountState } from '../model';

export const getAccountState = () => invoke<AccountState>('get_account_state');

export const createOfflineAccount = (username: string) =>
  invoke('create_offline_account', { username });

export const changeAccount = (id: Account['id']) =>
  invoke('change_account', { id });

export const logout = (id: Account['id']) => invoke('logout', { id });
