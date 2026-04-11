import React from 'react';
import {
  IonPage, IonContent, IonHeader, IonToolbar, IonTitle,
  IonList, IonItem, IonLabel, IonToggle, IonSpinner, IonButton,
} from '@ionic/react';
import { useParams } from 'react-router-dom';
import { useCalendars } from '../api/hooks';
import { api } from '../api/client';
import type { CalendarSource } from '@family-center/contracts';

const CalendarSelection: React.FC = () => {
  const { accountId } = useParams<{ accountId: string }>();
  const { data: calendars = [], isLoading, refetch } = useCalendars(accountId);

  const handleToggle = async (cal: CalendarSource, selected: boolean) => {
    await api.post('/google/calendars/select', {
      selections: [{ calendarSourceId: cal.id, isSelected: selected }],
    });
    refetch();
  };

  return (
    <IonPage>
      <IonHeader>
        <IonToolbar>
          <IonTitle>Select Calendars</IonTitle>
        </IonToolbar>
      </IonHeader>
      <IonContent>
        {isLoading ? (
          <div style={{ display: 'flex', justifyContent: 'center', padding: '40px' }}>
            <IonSpinner />
          </div>
        ) : (
          <IonList>
            {calendars.map((cal) => (
              <IonItem key={cal.id}>
                <div
                  slot="start"
                  style={{
                    width: 12, height: 12, borderRadius: '50%',
                    background: cal.colorHex ?? '#4A90D9', flexShrink: 0,
                  }}
                />
                <IonLabel>
                  <h2>{cal.name}</h2>
                  {cal.description && <p>{cal.description}</p>}
                  <p style={{ fontSize: '11px' }}>{cal.accessRole}</p>
                </IonLabel>
                <IonToggle
                  slot="end"
                  checked={cal.isSelected}
                  onIonChange={(e) => handleToggle(cal, e.detail.checked)}
                />
              </IonItem>
            ))}
          </IonList>
        )}
      </IonContent>
    </IonPage>
  );
};

export default CalendarSelection;
