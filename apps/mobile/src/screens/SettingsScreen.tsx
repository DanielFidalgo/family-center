import React from 'react';
import {
  IonPage, IonContent, IonHeader, IonToolbar, IonTitle,
  IonList, IonItem, IonLabel, IonToggle, IonButton, IonSpinner,
  IonNote, IonItemDivider,
} from '@ionic/react';
import { useHistory } from 'react-router-dom';
import { useSettings, useUpdateSettings, useRunSync } from '../api/hooks';
import DedupeSettingsPanel from '../components/settings/DedupeSettingsPanel';
import type { DedupeMode } from '@family-center/contracts';

const SettingsScreen: React.FC = () => {
  const history = useHistory();
  const { data: settings, isLoading } = useSettings();
  const updateMutation = useUpdateSettings();
  const syncMutation = useRunSync();

  const update = (changes: Parameters<typeof updateMutation.mutate>[0]) => {
    updateMutation.mutate(changes);
  };

  if (isLoading) {
    return (
      <IonPage>
        <IonContent>
          <div style={{ display: 'flex', justifyContent: 'center', padding: '40px' }}>
            <IonSpinner />
          </div>
        </IonContent>
      </IonPage>
    );
  }

  return (
    <IonPage>
      <IonHeader>
        <IonToolbar>
          <IonTitle>Settings</IonTitle>
        </IonToolbar>
      </IonHeader>
      <IonContent>
        <IonList>
          <IonItemDivider>
            <IonLabel>People & Calendars</IonLabel>
          </IonItemDivider>
          <IonItem button onClick={() => history.push('/people')}>
            <IonLabel>Manage people</IonLabel>
            <IonNote slot="end">Lanes</IonNote>
          </IonItem>
          <IonItem button onClick={() => history.push('/google-accounts')}>
            <IonLabel>Google accounts</IonLabel>
            <IonNote slot="end">Connect</IonNote>
          </IonItem>

          <IonItemDivider>
            <IonLabel>Display</IonLabel>
          </IonItemDivider>
          <IonItem>
            <IonLabel>Week starts Monday</IonLabel>
            <IonToggle
              slot="end"
              checked={settings?.weekStartsMonday ?? true}
              onIonChange={(e) => update({ weekStartsMonday: e.detail.checked })}
            />
          </IonItem>

          <IonItemDivider>
            <IonLabel>Duplicate handling</IonLabel>
          </IonItemDivider>
          <DedupeSettingsPanel
            value={(settings?.dedupeMode as DedupeMode) ?? 'exact_only'}
            onChange={(mode) => update({ dedupeMode: mode })}
          />

          <IonItemDivider>
            <IonLabel>Sync</IonLabel>
          </IonItemDivider>
          <IonItem>
            <IonButton
              expand="block"
              fill="outline"
              onClick={() => syncMutation.mutate({ forceFullSync: true })}
              disabled={syncMutation.isPending}
            >
              {syncMutation.isPending ? <IonSpinner name="crescent" /> : 'Force Full Sync'}
            </IonButton>
          </IonItem>
        </IonList>
      </IonContent>
    </IonPage>
  );
};

export default SettingsScreen;
