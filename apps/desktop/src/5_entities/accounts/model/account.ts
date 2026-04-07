export interface Account {
  id: string;
  username: string;
  active: boolean;
  accountType: AccountType;
}

export type AccountType = 'offline' | 'online';
