import { UUID, Timestamps } from './common';

export interface Household extends Timestamps {
  id: UUID;
  name: string;
}
