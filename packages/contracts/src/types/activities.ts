import { UUID, Timestamps } from './common';

export type RecurrenceFreq = 'daily' | 'weekly' | 'monthly' | 'yearly';

export interface LocalActivity extends Timestamps {
  id: UUID;
  householdId: UUID;
  personId?: UUID;          // null = shared lane
  title: string;
  description?: string;
  color?: string;
  startAt?: string;         // ISO 8601; null for all-day or time-of-day
  endAt?: string;
  isAllDay: boolean;
  recurrenceRule?: LocalRecurrenceRule;
}

export interface LocalRecurrenceRule {
  id: UUID;
  localActivityId: UUID;
  freq: RecurrenceFreq;
  interval: number;         // every N freq units
  byDayOfWeek?: number[];   // 0=Mon … 6=Sun
  byDayOfMonth?: number[];
  until?: string;           // ISO date, null = forever
  count?: number;
}

export interface CreateActivityRequest {
  personId?: UUID;
  title: string;
  description?: string;
  color?: string;
  startAt?: string;
  endAt?: string;
  isAllDay?: boolean;
  recurrence?: {
    freq: RecurrenceFreq;
    interval?: number;
    byDayOfWeek?: number[];
    byDayOfMonth?: number[];
    until?: string;
    count?: number;
  };
}

export interface UpdateActivityRequest {
  personId?: UUID | null;
  title?: string;
  description?: string;
  color?: string;
  startAt?: string;
  endAt?: string;
  isAllDay?: boolean;
  recurrence?: {
    freq: RecurrenceFreq;
    interval?: number;
    byDayOfWeek?: number[];
    byDayOfMonth?: number[];
    until?: string;
    count?: number;
  } | null;
}
