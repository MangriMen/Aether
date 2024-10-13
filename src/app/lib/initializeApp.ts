import { initializeState } from '@/entities/minecraft';

export const initializeApp = async () => {
  await initializeState();
};
