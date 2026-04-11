import { UUID, Timestamps } from './common';

export interface GoogleAccount extends Timestamps {
  id: UUID;
  householdId: UUID;
  email: string;
  displayName?: string;
  avatarUrl?: string;
  accessToken?: string;   // omitted in responses
  refreshToken?: string;  // omitted in responses
  tokenExpiresAt?: string;
  isActive: boolean;
}

export interface GoogleAccountPublic {
  id: UUID;
  householdId: UUID;
  email: string;
  displayName?: string;
  avatarUrl?: string;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface CalendarSource extends Timestamps {
  id: UUID;
  googleAccountId: UUID;
  calendarId: string;      // Google's calendar ID
  name: string;
  description?: string;
  colorHex?: string;
  isSelected: boolean;     // user chose to include this
  accessRole: string;      // owner | writer | reader | freeBusyReader
}

export interface SelectCalendarsRequest {
  selections: Array<{
    calendarSourceId: UUID;
    isSelected: boolean;
  }>;
}

export interface ConnectGoogleStartResponse {
  authUrl: string;
  state: string;
}
