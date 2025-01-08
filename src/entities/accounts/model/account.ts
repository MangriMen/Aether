export interface Account {
  id: string;
  active: boolean;
  username: string;
  accountType: AccountType;
}

export type AccountType = 'offline' | 'online';
