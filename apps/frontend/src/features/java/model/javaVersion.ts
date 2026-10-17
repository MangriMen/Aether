export interface JavaVersion {
  majorVersion: number;
  version: string | undefined;
  path: string | undefined;
}

export type StrictJavaVersion = {
  [K in keyof JavaVersion]: NonNullable<JavaVersion[K]>;
};
export type JavaTestStatus =
  'idle' | 'testing' | 'valid' | 'error' | 'version-mismatch';
