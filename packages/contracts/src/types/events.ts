import { UUID, Timestamps } from './common';
import type { ActivityCompletion, LocalActivity } from './activities';

export type DedupeMode = 'show_all' | 'exact_only' | 'strong' | 'probable';

export type DupeTier = 'exact' | 'strong' | 'probable';

export interface SourceEvent extends Timestamps {
  id: UUID;
  calendarSourceId: UUID;
  googleEventId: string;
  iCalUID?: string;
  title: string;
  description?: string;
  location?: string;
  startAt: string;          // ISO 8601
  endAt: string;
  isAllDay: boolean;
  recurrenceRule?: string;  // RRULE string
  recurringEventId?: string;
  organizer?: string;       // email
  attendees?: string[];     // emails
  rawJson: string;          // original Google API payload
  syncedAt: string;
}

export interface MergedEventGroup extends Timestamps {
  id: UUID;
  householdId: UUID;
  canonicalTitle: string;
  canonicalStart: string;
  canonicalEnd: string;
  isAllDay: boolean;
  personId?: UUID;          // null = shared lane
  laneOverride?: UUID;      // manual override
  dupeTier?: DupeTier;
  sources: MergedEventSource[];
}

export interface MergedEventSource {
  id: UUID;
  mergedEventGroupId: UUID;
  sourceEventId: UUID;
  isPrimary: boolean;
  sourceEvent?: SourceEvent;
}

export interface ScheduleResponse {
  events: MergedEventGroup[];
  localActivities: LocalActivity[];
  completions: ActivityCompletion[];
  start: string;
  end: string;
}
