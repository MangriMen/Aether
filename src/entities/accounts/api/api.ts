import { invoke } from '@tauri-apps/api/core';

import type { Account } from '../model';

const PLUGIN_PREFIX = 'plugin:auth|';

export const getAccounts = () =>
  invoke<Account[]>(`${PLUGIN_PREFIX}get_accounts`);

export const createOfflineAccount = (username: string) =>
  invoke<string>(`${PLUGIN_PREFIX}create_offline_account`, { username });

export const changeAccount = (id: Account['id']) =>
  invoke(`${PLUGIN_PREFIX}change_account`, { id });

export const logout = (id: Account['id']) =>
  invoke(`${PLUGIN_PREFIX}logout`, { id });
