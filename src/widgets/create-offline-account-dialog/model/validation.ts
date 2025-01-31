import { z } from 'zod';

export type CreateOfflineAccountFormValues = {
  username: string;
};

export const CreateOfflineAccountFormSchema = z.object({
  username: z.string().min(1, { message: 'Username is required' }),
});
