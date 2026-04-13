import React, { useState } from 'react';
import { IonSpinner } from '@ionic/react';
import type { LocalActivity, CreateActivityRequest, RecurrenceFreq, ChoreCategory } from '@family-center/contracts';
import type { Person } from '@family-center/contracts';

interface Props {
  initial?: LocalActivity;
  people: Person[];
  onSave: (data: CreateActivityRequest) => void;
  isSaving?: boolean;
}

const DAY_NAMES = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

type ChorePreset = 'none' | 'daily' | 'weekdays' | 'weekends' | 'custom';

function presetToRecurrence(preset: ChorePreset, customDays: number[]) {
  if (preset === 'none') return undefined;
  if (preset === 'daily') return { freq: 'daily' as RecurrenceFreq, interval: 1 };
  if (preset === 'weekdays') return { freq: 'weekly' as RecurrenceFreq, interval: 1, byDayOfWeek: [0, 1, 2, 3, 4] };
  if (preset === 'weekends') return { freq: 'weekly' as RecurrenceFreq, interval: 1, byDayOfWeek: [5, 6] };
  if (preset === 'custom') return customDays.length > 0
    ? { freq: 'weekly' as RecurrenceFreq, interval: 1, byDayOfWeek: customDays }
    : undefined;
  return undefined;
}

const LocalActivityEditor: React.FC<Props> = ({ initial, people, onSave, isSaving = false }) => {
  const [title, setTitle] = useState(initial?.title ?? '');
  const [personId, setPersonId] = useState<string>(initial?.personId ?? '');
  const [isAllDay, setIsAllDay] = useState(initial?.isAllDay ?? true);
  const [startAt, setStartAt] = useState(initial?.startAt ?? '');
  const [endAt, setEndAt] = useState(initial?.endAt ?? '');
  const [category, setCategory] = useState<ChoreCategory | ''>(initial?.category ?? '');
  const [isTimeBound, setIsTimeBound] = useState(initial?.isTimeBound ?? false);
  const [isChore, setIsChore] = useState(!!initial?.recurrenceRule);
  const [chorePreset, setChorePreset] = useState<ChorePreset>(() => {
    if (!initial?.recurrenceRule) return 'daily';
    const r = initial.recurrenceRule;
    if (r.freq === 'daily') return 'daily';
    if (r.freq === 'weekly' && r.byDayOfWeek?.join(',') === '0,1,2,3,4') return 'weekdays';
    if (r.freq === 'weekly' && r.byDayOfWeek?.join(',') === '5,6') return 'weekends';
    return 'custom';
  });
  const [customDays, setCustomDays] = useState<number[]>(
    initial?.recurrenceRule?.byDayOfWeek ?? []
  );

  const toggleDay = (d: number) => {
    setCustomDays((prev) =>
      prev.includes(d) ? prev.filter((x) => x !== d) : [...prev, d]
    );
  };

  const handleSave = () => {
    if (!title.trim()) return;
    const recurrence = isChore ? presetToRecurrence(chorePreset, customDays) : undefined;
    const data: CreateActivityRequest = {
      title: title.trim(),
      personId: personId || undefined,
      isAllDay,
      startAt: startAt || undefined,
      endAt: endAt || undefined,
      category: category || undefined,
      isTimeBound,
      recurrence,
    };
    onSave(data);
  };

  const selectedPerson = people.find((p) => p.id === personId);

  return (
    <div style={styles.container}>
      {/* Title */}
      <div style={styles.field}>
        <label style={styles.label}>Activity name</label>
        <input
          style={styles.input}
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder="e.g. Brush teeth, Swimming, Doctor visit"
          autoFocus
        />
      </div>

      {/* Assign to person */}
      <div style={styles.field}>
        <label style={styles.label}>Assign to</label>
        <div style={styles.personRow}>
          <button
            style={{
              ...styles.personChip,
              ...(personId === '' ? styles.personChipActive : {}),
              borderColor: personId === '' ? 'var(--fc-shared-lane-color)' : 'var(--fc-border)',
            }}
            onClick={() => setPersonId('')}
          >
            <span style={{
              ...styles.personDot,
              background: 'var(--fc-shared-lane-color)',
            }} />
            Shared
          </button>
          {people.map((p) => (
            <button
              key={p.id}
              style={{
                ...styles.personChip,
                ...(personId === p.id ? styles.personChipActive : {}),
                borderColor: personId === p.id ? p.color : 'var(--fc-border)',
              }}
              onClick={() => setPersonId(p.id)}
            >
              <span style={{ ...styles.personDot, background: p.color }} />
              {p.name}
            </button>
          ))}
        </div>
      </div>

      {/* Chore toggle */}
      <div style={styles.choreToggleRow}>
        <div>
          <div style={styles.choreToggleLabel}>Repeating chore</div>
          <div style={styles.choreToggleSub}>Shows up on a schedule automatically</div>
        </div>
        <button
          style={{
            ...styles.toggle,
            background: isChore ? 'var(--fc-accent)' : 'var(--fc-border-strong)',
          }}
          onClick={() => setIsChore((v) => !v)}
        >
          <span style={{
            ...styles.toggleThumb,
            transform: isChore ? 'translateX(22px)' : 'translateX(2px)',
          }} />
        </button>
      </div>

      {/* Chore recurrence presets */}
      {isChore && (
        <div style={styles.recurrenceBlock}>
          <label style={styles.label}>Repeat</label>
          <div style={styles.presetRow}>
            {(['daily', 'weekdays', 'weekends', 'custom'] as ChorePreset[]).map((preset) => {
              const labels: Record<ChorePreset, string> = {
                none: '', daily: 'Every day', weekdays: 'Weekdays', weekends: 'Weekends', custom: 'Custom…',
              };
              return (
                <button
                  key={preset}
                  style={{
                    ...styles.presetChip,
                    ...(chorePreset === preset ? styles.presetChipActive : {}),
                  }}
                  onClick={() => setChorePreset(preset)}
                >
                  {labels[preset]}
                </button>
              );
            })}
          </div>

          {chorePreset === 'custom' && (
            <div style={styles.dayPicker}>
              {DAY_NAMES.map((day, i) => (
                <button
                  key={i}
                  style={{
                    ...styles.dayBtn,
                    ...(customDays.includes(i) ? styles.dayBtnActive : {}),
                  }}
                  onClick={() => toggleDay(i)}
                >
                  {day}
                </button>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Category */}
      {isChore && (
        <div style={styles.field}>
          <label style={styles.label}>Category</label>
          <div style={styles.presetRow}>
            {(['cleaning', 'kitchen', 'laundry', 'hygiene', 'pets', 'homework', 'exercise', 'errands', 'other'] as ChoreCategory[]).map((cat) => (
              <button
                key={cat}
                style={{
                  ...styles.presetChip,
                  ...(category === cat ? styles.presetChipActive : {}),
                }}
                onClick={() => setCategory(category === cat ? '' : cat)}
              >
                {cat.charAt(0).toUpperCase() + cat.slice(1)}
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Time-bound toggle */}
      {isChore && (
        <div style={styles.choreToggleRow}>
          <div>
            <div style={styles.choreToggleLabel}>Time-bound</div>
            <div style={styles.choreToggleSub}>Must be done at a specific time, not anytime</div>
          </div>
          <button
            style={{
              ...styles.toggle,
              background: isTimeBound ? 'var(--fc-accent)' : 'var(--fc-border-strong)',
            }}
            onClick={() => setIsTimeBound((v) => !v)}
          >
            <span style={{
              ...styles.toggleThumb,
              transform: isTimeBound ? 'translateX(22px)' : 'translateX(2px)',
            }} />
          </button>
        </div>
      )}

      {/* Time (optional, shown when not all-day) */}
      <div style={styles.timeSection}>
        <div style={styles.timeLabelRow}>
          <label style={styles.label}>Time</label>
          <button
            style={{
              ...styles.allDayBtn,
              background: isAllDay ? 'var(--fc-blue-dim)' : 'transparent',
              color: isAllDay ? 'var(--fc-blue)' : 'var(--fc-text-secondary)',
              borderColor: isAllDay ? 'var(--fc-blue)' : 'var(--fc-border)',
            }}
            onClick={() => setIsAllDay((v) => !v)}
          >
            All day
          </button>
        </div>

        {!isAllDay && (
          <div style={styles.timeRow}>
            <div style={styles.timeField}>
              <label style={styles.timeSublabel}>Start</label>
              <input
                type="datetime-local"
                style={styles.timeInput}
                value={startAt ? new Date(startAt).toISOString().slice(0, 16) : ''}
                onChange={(e) => setStartAt(e.target.value ? new Date(e.target.value).toISOString() : '')}
              />
            </div>
            <div style={styles.timeField}>
              <label style={styles.timeSublabel}>End</label>
              <input
                type="datetime-local"
                style={styles.timeInput}
                value={endAt ? new Date(endAt).toISOString().slice(0, 16) : ''}
                onChange={(e) => setEndAt(e.target.value ? new Date(e.target.value).toISOString() : '')}
              />
            </div>
          </div>
        )}
      </div>

      {/* Save */}
      <button
        style={{
          ...styles.saveBtn,
          opacity: !title.trim() || isSaving ? 0.5 : 1,
        }}
        onClick={handleSave}
        disabled={!title.trim() || isSaving}
      >
        {isSaving
          ? <IonSpinner name="crescent" style={{ width: 18, height: 18, color: '#000' }} />
          : isChore
          ? `Save chore${chorePreset === 'daily' ? ' (every day)' : chorePreset === 'weekdays' ? ' (weekdays)' : chorePreset === 'weekends' ? ' (weekends)' : ''}`
          : 'Save activity'
        }
      </button>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    gap: '20px',
    padding: '20px 16px',
    fontFamily: 'var(--fc-font-body)',
  },
  field: {
    display: 'flex',
    flexDirection: 'column',
    gap: '8px',
  },
  label: {
    fontSize: '11px',
    fontWeight: 600,
    textTransform: 'uppercase',
    letterSpacing: '0.06em',
    color: 'var(--fc-text-secondary)',
    fontFamily: 'var(--fc-font-body)',
  },
  input: {
    background: 'var(--fc-bg-card)',
    border: '1px solid var(--fc-border)',
    borderRadius: '8px',
    padding: '12px 14px',
    color: 'var(--fc-text-primary)',
    fontSize: '16px',
    fontFamily: 'var(--fc-font-body)',
    outline: 'none',
    width: '100%',
    boxSizing: 'border-box',
  },
  personRow: {
    display: 'flex',
    flexWrap: 'wrap',
    gap: '8px',
  },
  personChip: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    padding: '8px 14px',
    borderRadius: '20px',
    background: 'var(--fc-bg-card)',
    border: '1.5px solid var(--fc-border)',
    color: 'var(--fc-text-primary)',
    fontSize: '13px',
    fontFamily: 'var(--fc-font-body)',
    fontWeight: 500,
    cursor: 'pointer',
    outline: 'none',
    transition: 'border-color 0.15s, background 0.15s',
    minHeight: '40px',
  },
  personChipActive: {
    background: 'var(--fc-bg-elevated)',
  },
  personDot: {
    width: '8px',
    height: '8px',
    borderRadius: '50%',
    flexShrink: 0,
  },
  choreToggleRow: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    background: 'var(--fc-bg-card)',
    borderRadius: '10px',
    padding: '14px 16px',
    border: '1px solid var(--fc-border)',
    gap: '12px',
  },
  choreToggleLabel: {
    fontSize: '15px',
    fontWeight: 600,
    color: 'var(--fc-text-primary)',
    fontFamily: 'var(--fc-font-body)',
    marginBottom: '2px',
  },
  choreToggleSub: {
    fontSize: '12px',
    color: 'var(--fc-text-secondary)',
    fontFamily: 'var(--fc-font-body)',
  },
  toggle: {
    position: 'relative',
    width: '46px',
    height: '26px',
    borderRadius: '13px',
    border: 'none',
    cursor: 'pointer',
    flexShrink: 0,
    padding: 0,
    transition: 'background 0.2s',
    outline: 'none',
  },
  toggleThumb: {
    position: 'absolute',
    top: '3px',
    width: '20px',
    height: '20px',
    borderRadius: '50%',
    background: '#fff',
    transition: 'transform 0.2s',
    boxShadow: '0 1px 4px rgba(0,0,0,0.3)',
  },
  recurrenceBlock: {
    display: 'flex',
    flexDirection: 'column',
    gap: '10px',
    padding: '14px 16px',
    background: 'var(--fc-bg-card)',
    borderRadius: '10px',
    border: '1px solid var(--fc-border)',
  },
  presetRow: {
    display: 'flex',
    gap: '8px',
    flexWrap: 'wrap',
  },
  presetChip: {
    padding: '8px 14px',
    borderRadius: '20px',
    background: 'var(--fc-bg)',
    border: '1.5px solid var(--fc-border)',
    color: 'var(--fc-text-secondary)',
    fontSize: '13px',
    fontFamily: 'var(--fc-font-body)',
    fontWeight: 500,
    cursor: 'pointer',
    outline: 'none',
    transition: 'all 0.15s',
    minHeight: '40px',
  },
  presetChipActive: {
    background: 'var(--fc-accent-dim)',
    borderColor: 'var(--fc-accent)',
    color: 'var(--fc-accent)',
    fontWeight: 600,
  },
  dayPicker: {
    display: 'flex',
    gap: '6px',
    flexWrap: 'wrap',
    paddingTop: '4px',
  },
  dayBtn: {
    width: '44px',
    height: '44px',
    borderRadius: '8px',
    background: 'var(--fc-bg)',
    border: '1.5px solid var(--fc-border)',
    color: 'var(--fc-text-secondary)',
    fontSize: '12px',
    fontWeight: 600,
    fontFamily: 'var(--fc-font-body)',
    cursor: 'pointer',
    outline: 'none',
    transition: 'all 0.15s',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  },
  dayBtnActive: {
    background: 'var(--fc-accent-dim)',
    borderColor: 'var(--fc-accent)',
    color: 'var(--fc-accent)',
  },
  timeSection: {
    display: 'flex',
    flexDirection: 'column',
    gap: '10px',
  },
  timeLabelRow: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  allDayBtn: {
    padding: '5px 12px',
    borderRadius: '6px',
    border: '1.5px solid',
    fontSize: '12px',
    fontFamily: 'var(--fc-font-body)',
    fontWeight: 600,
    cursor: 'pointer',
    outline: 'none',
    transition: 'all 0.15s',
  },
  timeRow: {
    display: 'flex',
    gap: '10px',
  },
  timeField: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    gap: '6px',
  },
  timeSublabel: {
    fontSize: '11px',
    fontWeight: 600,
    textTransform: 'uppercase',
    letterSpacing: '0.06em',
    color: 'var(--fc-text-muted)',
    fontFamily: 'var(--fc-font-body)',
  },
  timeInput: {
    background: 'var(--fc-bg-card)',
    border: '1px solid var(--fc-border)',
    borderRadius: '8px',
    padding: '10px 12px',
    color: 'var(--fc-text-primary)',
    fontSize: '14px',
    fontFamily: 'var(--fc-font-body)',
    outline: 'none',
    width: '100%',
    boxSizing: 'border-box',
    colorScheme: 'dark',
  },
  saveBtn: {
    width: '100%',
    padding: '15px',
    borderRadius: '10px',
    background: 'var(--fc-accent)',
    border: 'none',
    color: '#000',
    fontSize: '15px',
    fontWeight: 700,
    fontFamily: 'var(--fc-font-display)',
    cursor: 'pointer',
    letterSpacing: '0.01em',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    transition: 'opacity 0.15s',
    minHeight: '52px',
  },
};

export default LocalActivityEditor;
