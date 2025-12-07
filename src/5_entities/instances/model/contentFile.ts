import type { ContentType } from './contentType';

export interface ContentFile {
  contentPath: string;
  contentType: ContentType;
  disabled: boolean;
  filename: string;
  hash: string;
  instanceRelativePath: string;
  name?: string;
  size: number;
  update: Record<string, Record<string, unknown>>;
}
