import { createResource } from 'solid-js';

import { getUpdate } from '../api';

export const updateResource = createResource(() => getUpdate());
