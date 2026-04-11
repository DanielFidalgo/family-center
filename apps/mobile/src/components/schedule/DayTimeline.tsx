import React, { useRef } from 'react';
import type { Person, MergedEventGroup, LocalActivity, Settings } from '@family-center/contracts';
import './DayTimeline.css';

interface Props {
  people: Person[];
  events: MergedEventGroup[];
  activities: LocalActivity[];
  date: Date;
  settings?: Settings;
  onEventClick?: (event: MergedEventGroup) => void;
  onActivityClick?: (activity: LocalActivity) => void;
}

const HOURS = Array.from({ length: 24 }, (_, i) => i);
const HOUR_HEIGHT = 80; // px per hour

function positionForTime(dt: string, baseDate: Date): { top: number; height: number } | null {
  const d = new Date(dt);
  const startOfDay = new Date(baseDate);
  startOfDay.setHours(0, 0, 0, 0);
  const offset = (d.getTime() - startOfDay.getTime()) / 3600000;
  if (offset < 0 || offset > 24) return null;
  return { top: offset * HOUR_HEIGHT, height: HOUR_HEIGHT };
}

function eventPosition(start: string, end: string, baseDate: Date) {
  const startPos = positionForTime(start, baseDate);
  if (!startPos) return null;
  const durationHours = (new Date(end).getTime() - new Date(start).getTime()) / 3600000;
  return {
    top: startPos.top,
    height: Math.max(32, durationHours * HOUR_HEIGHT),
  };
}

const DayTimeline: React.FC<Props> = ({
  people,
  events,
  activities,
  date,
  settings,
  onEventClick,
  onActivityClick,
}) => {
  const scrollRef = useRef<HTMLDivElement>(null);

  const eventsForPerson = (personId: string | null) =>
    events.filter((e) => (personId ? e.personId === personId : !e.personId));

  const activitiesForPerson = (personId: string | null) =>
    activities.filter((a) => (personId ? a.personId === personId : !a.personId));

  const lanes = [
    { person: null as Person | null, label: 'Shared', color: 'var(--fc-shared-lane-color)' },
    ...people.map((p) => ({ person: p, label: p.name, color: p.color })),
  ];

  return (
    <div className="day-timeline">
      {/* Column headers */}
      <div className="day-timeline__headers">
        <div className="day-timeline__time-gutter" />
        {lanes.map(({ person, label, color }) => (
          <div
            key={person?.id ?? 'shared'}
            className="day-timeline__col-header"
            style={{ borderTop: `3px solid ${color}` }}
          >
            <span className="day-timeline__col-dot" style={{ background: color }} />
            {label}
          </div>
        ))}
      </div>

      {/* Scrollable time grid */}
      <div className="day-timeline__scroll" ref={scrollRef}>
        <div className="day-timeline__grid" style={{ height: `${HOUR_HEIGHT * 24}px` }}>
          {/* Time labels */}
          <div className="day-timeline__time-col">
            {HOURS.map((h) => (
              <div
                key={h}
                className="day-timeline__time-label"
                style={{ top: `${h * HOUR_HEIGHT}px` }}
              >
                {h === 0 ? '12am' : h < 12 ? `${h}am` : h === 12 ? '12pm' : `${h - 12}pm`}
              </div>
            ))}
          </div>

          {/* Lane columns */}
          {lanes.map(({ person, color }) => {
            const pid = person?.id ?? null;
            const laneEvents = eventsForPerson(pid);
            const laneActivities = activitiesForPerson(pid);

            return (
              <div key={pid ?? 'shared'} className="day-timeline__lane-col">
                {/* Hour grid lines */}
                {HOURS.map((h) => (
                  <div
                    key={h}
                    className="day-timeline__hour-line"
                    style={{ top: `${h * HOUR_HEIGHT}px` }}
                  />
                ))}

                {/* Events */}
                {laneEvents.map((ev) => {
                  const pos = eventPosition(ev.canonicalStart, ev.canonicalEnd, date);
                  if (!pos) return null;
                  return (
                    <div
                      key={ev.id}
                      className="day-timeline__event"
                      style={{
                        top: `${pos.top}px`,
                        height: `${pos.height}px`,
                        borderLeft: `3px solid ${color}`,
                        cursor: onEventClick ? 'pointer' : 'default',
                      }}
                      onClick={onEventClick ? () => onEventClick(ev) : undefined}
                    >
                      <div className="day-timeline__event-title">{ev.canonicalTitle}</div>
                      <div className="day-timeline__event-time">
                        {new Date(ev.canonicalStart).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                      </div>
                    </div>
                  );
                })}

                {/* Activities */}
                {laneActivities.filter((a) => a.startAt).map((act) => {
                  const pos = eventPosition(act.startAt!, act.endAt ?? act.startAt!, date);
                  if (!pos) return null;
                  return (
                    <div
                      key={act.id}
                      className="day-timeline__event day-timeline__event--activity"
                      style={{
                        top: `${pos.top}px`,
                        height: `${pos.height}px`,
                        borderLeft: `3px solid ${act.color ?? color}`,
                        cursor: onActivityClick ? 'pointer' : 'default',
                      }}
                      onClick={onActivityClick ? () => onActivityClick(act) : undefined}
                    >
                      <div className="day-timeline__event-title">{act.title}</div>
                    </div>
                  );
                })}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};

export default DayTimeline;
