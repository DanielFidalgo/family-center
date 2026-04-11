import { UUID, Timestamps } from './common';

export interface Person extends Timestamps {
  id: UUID;
  householdId: UUID;
  name: string;
  color: string;       // hex color for lane
  avatarUrl?: string;
  sortOrder: number;
  isActive: boolean;
}

export interface CreatePersonRequest {
  name: string;
  color: string;
  avatarUrl?: string;
  sortOrder?: number;
}

export interface UpdatePersonRequest {
  name?: string;
  color?: string;
  avatarUrl?: string;
  sortOrder?: number;
  isActive?: boolean;
}
