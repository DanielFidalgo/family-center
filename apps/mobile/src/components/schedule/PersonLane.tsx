import React from 'react';
import type { Person, MergedEventGroup, LocalActivity } from '@family-center/contracts';
import EventCard from './EventCard';
import './PersonLane.css';

interface Props {
  person: Person | null; // null = shared lane
  events: MergedEventGroup[];
  activities: LocalActivity[];
  date: Date;
  dedupeMode?: string;
  onEventClick?: (event: MergedEventGroup) => void;
  onActivityClick?: (activity: LocalActivity) => void;
}

const PersonLane: React.FC<Props> = ({
  person,
  events,
  activities,
  date,
  dedupeMode = 'exact_only',
  onEventClick,
  onActivityClick,
}) => {
  const laneColor = person?.color ?? 'var(--fc-shared-lane-color)';
  const laneName = person?.name ?? 'Shared';

  // Filter duplicates per dedupeMode
  const visibleEvents = events.filter((e) => {
    if (dedupeMode === 'show_all') return true;
    if (dedupeMode === 'exact_only') return e.dupeTier !== 'exact' || e.sources?.[0]?.isPrimary;
    if (dedupeMode === 'strong') return !e.dupeTier || ['probable'].includes(e.dupeTier ?? '') || false;
    if (dedupeMode === 'probable') return !e.dupeTier;
    return true;
  });

  const allItems = [
    ...visibleEvents.map((e) => ({ type: 'event' as const, key: e.id, startAt: e.canonicalStart, data: e })),
    ...activities.map((a) => ({ type: 'activity' as const, key: a.id, startAt: a.startAt ?? '', data: a })),
  ].sort((a, b) => new Date(a.startAt).getTime() - new Date(b.startAt).getTime());

  return (
    <div className="person-lane">
      <div className="person-lane__header" style={{ borderLeft: `3px solid ${laneColor}` }}>
        <span
          className="person-lane__dot"
          style={{ background: laneColor }}
        />
        <span className="person-lane__name">{laneName}</span>
        <span className="person-lane__count">{allItems.length}</span>
      </div>
      <div className="person-lane__events">
        {allItems.length === 0 ? (
          <div className="person-lane__empty">No events</div>
        ) : (
          allItems.map((item) =>
            item.type === 'event' ? (
              <EventCard
                key={item.key}
                item={{ type: 'merged', event: item.data as MergedEventGroup }}
                color={laneColor}
                compact
                onClick={onEventClick ? () => onEventClick(item.data as MergedEventGroup) : undefined}
              />
            ) : (
              <EventCard
                key={item.key}
                item={{ type: 'activity', activity: item.data as LocalActivity }}
                color={laneColor}
                compact
                onClick={onActivityClick ? () => onActivityClick(item.data as LocalActivity) : undefined}
              />
            )
          )
        )}
      </div>
    </div>
  );
};

export default PersonLane;
