import { z } from 'zod';

export type CreateOfflineAccountFormValues = {
  username: string;
};

export type CreateOfflineAccountFormSchemaErrors = 'usernameError';

export const CreateOfflineAccountFormSchema = z.object({
  username: z.string().regex(/^\w{1,16}$/, {
    message: 'usernameError',
  }),
});
