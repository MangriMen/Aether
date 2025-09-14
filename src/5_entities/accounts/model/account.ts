export interface Account {
  accountType: AccountType;
  active: boolean;
  id: string;
  username: string;
}

export type AccountType = 'offline' | 'online';
