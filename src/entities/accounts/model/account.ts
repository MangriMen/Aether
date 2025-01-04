export interface Account {
  id: string;
  username: string;
}

export interface AccountState {
  default?: string;
  accounts: Account[];
}

export type AccountType = 'offline' | 'online';
