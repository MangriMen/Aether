import { invoke } from '@tauri-apps/api/core';

import type { Account } from '../model';

const PLUGIN_PREFIX = 'plugin:auth|';

export const listAccountsRaw = () =>
  invoke<Account[]>(`${PLUGIN_PREFIX}get_accounts`);

export const createOfflineAccountRaw = (username: string) =>
  invoke<string>(`${PLUGIN_PREFIX}create_offline_account`, { username });

export const changeAccountRaw = (id: Account['id']) =>
  invoke(`${PLUGIN_PREFIX}change_account`, { id });

export const logoutRaw = (id: Account['id']) =>
  invoke(`${PLUGIN_PREFIX}logout`, { id });
