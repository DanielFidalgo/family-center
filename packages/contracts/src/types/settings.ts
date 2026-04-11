import { UUID, Timestamps } from './common';
import { DedupeMode } from './events';

export type DefaultView = 'day' | 'week';

export interface Settings extends Timestamps {
  householdId: UUID;
  defaultView: DefaultView;
  weekStartsMonday: boolean;
  dedupeMode: DedupeMode;
  displayTimezone: string;  // IANA tz name
}

export interface UpdateSettingsRequest {
  defaultView?: DefaultView;
  weekStartsMonday?: boolean;
  dedupeMode?: DedupeMode;
  displayTimezone?: string;
}
