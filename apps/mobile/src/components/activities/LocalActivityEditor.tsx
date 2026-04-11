import React, { useState, useEffect } from 'react';
import {
  IonItem, IonLabel, IonInput, IonTextarea, IonSelect, IonSelectOption,
  IonToggle, IonDatetime, IonButton, IonSpinner,
} from '@ionic/react';
import type { LocalActivity, CreateActivityRequest, RecurrenceFreq } from '@family-center/contracts';
import type { Person } from '@family-center/contracts';

interface Props {
  initial?: LocalActivity;
  people: Person[];
  onSave: (data: CreateActivityRequest) => void;
  isSaving?: boolean;
}

const LocalActivityEditor: React.FC<Props> = ({ initial, people, onSave, isSaving = false }) => {
  const [title, setTitle] = useState(initial?.title ?? '');
  const [description, setDescription] = useState(initial?.description ?? '');
  const [personId, setPersonId] = useState<string>(initial?.personId ?? '');
  const [isAllDay, setIsAllDay] = useState(initial?.isAllDay ?? false);
  const [startAt, setStartAt] = useState(initial?.startAt ?? '');
  const [endAt, setEndAt] = useState(initial?.endAt ?? '');
  const [isRecurring, setIsRecurring] = useState(!!initial?.recurrenceRule);
  const [freq, setFreq] = useState<RecurrenceFreq>('weekly');
  const [interval, setInterval] = useState(1);
  const [byDayOfWeek, setByDayOfWeek] = useState<number[]>([]);

  const DAY_NAMES = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

  const toggleDay = (dayNum: number) => {
    setByDayOfWeek((prev) =>
      prev.includes(dayNum) ? prev.filter((d) => d !== dayNum) : [...prev, dayNum]
    );
  };

  const handleSave = () => {
    const data: CreateActivityRequest = {
      title,
      description: description || undefined,
      personId: personId || undefined,
      isAllDay,
      startAt: startAt || undefined,
      endAt: endAt || undefined,
      recurrence: isRecurring ? {
        freq,
        interval,
        byDayOfWeek: byDayOfWeek.length > 0 ? byDayOfWeek : undefined,
      } : undefined,
    };
    onSave(data);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
      <IonItem>
        <IonLabel position="stacked">Title *</IonLabel>
        <IonInput
          value={title}
          onIonChange={(e) => setTitle(e.detail.value ?? '')}
          placeholder="e.g. Swim practice"
        />
      </IonItem>

      <IonItem>
        <IonLabel position="stacked">Description</IonLabel>
        <IonTextarea
          value={description}
          onIonChange={(e) => setDescription(e.detail.value ?? '')}
          rows={2}
          placeholder="Optional notes"
        />
      </IonItem>

      <IonItem>
        <IonLabel position="stacked">Assign to</IonLabel>
        <IonSelect
          value={personId}
          onIonChange={(e) => setPersonId(e.detail.value)}
          placeholder="Shared"
        >
          <IonSelectOption value="">Shared Lane</IonSelectOption>
          {people.map((p) => (
            <IonSelectOption key={p.id} value={p.id}>{p.name}</IonSelectOption>
          ))}
        </IonSelect>
      </IonItem>

      <IonItem>
        <IonLabel>All day</IonLabel>
        <IonToggle
          slot="end"
          checked={isAllDay}
          onIonChange={(e) => setIsAllDay(e.detail.checked)}
        />
      </IonItem>

      {!isAllDay && (
        <>
          <IonItem>
            <IonLabel position="stacked">Start time</IonLabel>
            <IonInput
              type="datetime-local"
              value={startAt ? new Date(startAt).toISOString().slice(0, 16) : ''}
              onIonChange={(e) => setStartAt(e.detail.value ? new Date(e.detail.value).toISOString() : '')}
            />
          </IonItem>
          <IonItem>
            <IonLabel position="stacked">End time</IonLabel>
            <IonInput
              type="datetime-local"
              value={endAt ? new Date(endAt).toISOString().slice(0, 16) : ''}
              onIonChange={(e) => setEndAt(e.detail.value ? new Date(e.detail.value).toISOString() : '')}
            />
          </IonItem>
        </>
      )}

      <IonItem>
        <IonLabel>Recurring</IonLabel>
        <IonToggle
          slot="end"
          checked={isRecurring}
          onIonChange={(e) => setIsRecurring(e.detail.checked)}
        />
      </IonItem>

      {isRecurring && (
        <>
          <IonItem>
            <IonLabel position="stacked">Frequency</IonLabel>
            <IonSelect value={freq} onIonChange={(e) => setFreq(e.detail.value)}>
              <IonSelectOption value="daily">Daily</IonSelectOption>
              <IonSelectOption value="weekly">Weekly</IonSelectOption>
              <IonSelectOption value="monthly">Monthly</IonSelectOption>
              <IonSelectOption value="yearly">Yearly</IonSelectOption>
            </IonSelect>
          </IonItem>

          <IonItem>
            <IonLabel position="stacked">Every N {freq === 'daily' ? 'days' : freq === 'weekly' ? 'weeks' : freq === 'monthly' ? 'months' : 'years'}</IonLabel>
            <IonInput
              type="number"
              value={interval}
              min={1}
              onIonChange={(e) => setInterval(parseInt(e.detail.value ?? '1', 10) || 1)}
            />
          </IonItem>

          {freq === 'weekly' && (
            <div style={{ padding: '12px 16px' }}>
              <IonLabel style={{ fontSize: '12px', color: '#a0a0b0', display: 'block', marginBottom: '8px' }}>
                Days of week
              </IonLabel>
              <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
                {DAY_NAMES.map((day, i) => (
                  <div
                    key={i}
                    onClick={() => toggleDay(i)}
                    style={{
                      padding: '8px 12px',
                      borderRadius: '6px',
                      background: byDayOfWeek.includes(i) ? '#4A90D9' : '#16213e',
                      color: byDayOfWeek.includes(i) ? '#fff' : '#a0a0b0',
                      cursor: 'pointer',
                      fontSize: '13px',
                      fontWeight: 600,
                      minWidth: '44px',
                      textAlign: 'center',
                      border: '1px solid #2a2a4a',
                    }}
                  >
                    {day}
                  </div>
                ))}
              </div>
            </div>
          )}
        </>
      )}

      <div style={{ padding: '16px' }}>
        <IonButton
          expand="block"
          onClick={handleSave}
          disabled={!title.trim() || isSaving}
        >
          {isSaving ? <IonSpinner name="crescent" /> : 'Save Activity'}
        </IonButton>
      </div>
    </div>
  );
};

export default LocalActivityEditor;
