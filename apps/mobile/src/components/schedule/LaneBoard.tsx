import React from 'react';
import type { Person, MergedEventGroup, LocalActivity, Settings } from '@family-center/contracts';
import PersonLane from './PersonLane';
import './LaneBoard.css';

interface Props {
  people: Person[];
  events: MergedEventGroup[];
  activities: LocalActivity[];
  date: Date;
  settings?: Settings;
  onEventClick?: (event: MergedEventGroup) => void;
  onActivityClick?: (activity: LocalActivity) => void;
}

const LaneBoard: React.FC<Props> = ({
  people,
  events,
  activities,
  date,
  settings,
  onEventClick,
  onActivityClick,
}) => {
  const dedupeMode = settings?.dedupeMode ?? 'exact_only';

  // Events and activities for a person
  const eventsForPerson = (personId: string | null) =>
    events.filter((e) => (personId ? e.personId === personId : !e.personId));

  const activitiesForPerson = (personId: string | null) =>
    activities.filter((a) => (personId ? a.personId === personId : !a.personId));

  return (
    <div className="lane-board">
      {/* Shared lane always first */}
      <PersonLane
        person={null}
        events={eventsForPerson(null)}
        activities={activitiesForPerson(null)}
        date={date}
        dedupeMode={dedupeMode}
        onEventClick={onEventClick}
        onActivityClick={onActivityClick}
      />
      {/* Person lanes */}
      {people.map((person) => (
        <PersonLane
          key={person.id}
          person={person}
          events={eventsForPerson(person.id)}
          activities={activitiesForPerson(person.id)}
          date={date}
          dedupeMode={dedupeMode}
          onEventClick={onEventClick}
          onActivityClick={onActivityClick}
        />
      ))}
    </div>
  );
};

export default LaneBoard;
