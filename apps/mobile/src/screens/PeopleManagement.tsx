import React, { useState } from 'react';
import {
  IonPage, IonContent, IonHeader, IonToolbar, IonTitle,
  IonList, IonItem, IonLabel, IonButton, IonIcon, IonInput,
  IonModal, IonSpinner, IonFab, IonFabButton, IonText,
} from '@ionic/react';
import { addOutline, createOutline } from 'ionicons/icons';
import { usePeople, useCreatePerson, useUpdatePerson } from '../api/hooks';
import type { Person } from '@family-center/contracts';

const COLORS = ['#4A90D9', '#E85555', '#50C878', '#F5A623', '#9B59B6', '#1ABC9C', '#E67E22', '#E91E63'];

const PeopleManagement: React.FC = () => {
  const { data: people = [], isLoading } = usePeople();
  const createMutation = useCreatePerson();
  const updateMutation = useUpdatePerson();

  const [showModal, setShowModal] = useState(false);
  const [editing, setEditing] = useState<Person | null>(null);
  const [name, setName] = useState('');
  const [color, setColor] = useState(COLORS[0]);

  const openCreate = () => {
    setEditing(null);
    setName('');
    setColor(COLORS[people.length % COLORS.length]);
    setShowModal(true);
  };

  const openEdit = (person: Person) => {
    setEditing(person);
    setName(person.name);
    setColor(person.color);
    setShowModal(true);
  };

  const handleSave = async () => {
    if (!name.trim()) return;
    if (editing) {
      await updateMutation.mutateAsync({ id: editing.id, name, color });
    } else {
      await createMutation.mutateAsync({ name, color, sortOrder: people.length });
    }
    setShowModal(false);
  };

  return (
    <IonPage>
      <IonHeader>
        <IonToolbar>
          <IonTitle>People & Lanes</IonTitle>
        </IonToolbar>
      </IonHeader>
      <IonContent>
        {isLoading ? (
          <div style={{ display: 'flex', justifyContent: 'center', padding: '40px' }}>
            <IonSpinner />
          </div>
        ) : (
          <IonList>
            {people.map((person) => (
              <IonItem key={person.id} button onClick={() => openEdit(person)}>
                <div
                  slot="start"
                  style={{
                    width: 16, height: 16, borderRadius: '50%',
                    background: person.color, flexShrink: 0,
                  }}
                />
                <IonLabel>{person.name}</IonLabel>
                <IonIcon icon={createOutline} slot="end" color="medium" />
              </IonItem>
            ))}
            {people.length === 0 && (
              <IonItem lines="none">
                <IonText color="medium">
                  <p style={{ textAlign: 'center', width: '100%' }}>
                    No people yet. Add someone to create a lane.
                  </p>
                </IonText>
              </IonItem>
            )}
          </IonList>
        )}

        <IonFab vertical="bottom" horizontal="end" slot="fixed">
          <IonFabButton onClick={openCreate}>
            <IonIcon icon={addOutline} />
          </IonFabButton>
        </IonFab>

        <IonModal isOpen={showModal} onDidDismiss={() => setShowModal(false)}>
          <IonHeader>
            <IonToolbar>
              <IonTitle>{editing ? 'Edit Person' : 'Add Person'}</IonTitle>
              <IonButton slot="end" fill="clear" onClick={() => setShowModal(false)}>
                Cancel
              </IonButton>
            </IonToolbar>
          </IonHeader>
          <IonContent>
            <div style={{ padding: '20px', display: 'flex', flexDirection: 'column', gap: '20px' }}>
              <IonItem>
                <IonLabel position="stacked">Name</IonLabel>
                <IonInput
                  value={name}
                  onIonChange={(e) => setName(e.detail.value ?? '')}
                  placeholder="Alice"
                  autofocus
                />
              </IonItem>

              <div>
                <IonLabel style={{ fontSize: '12px', color: '#a0a0b0', paddingLeft: '16px' }}>
                  Lane Color
                </IonLabel>
                <div style={{ display: 'flex', flexWrap: 'wrap', gap: '12px', padding: '12px 16px' }}>
                  {COLORS.map((c) => (
                    <div
                      key={c}
                      onClick={() => setColor(c)}
                      style={{
                        width: 44, height: 44, borderRadius: '50%', background: c, cursor: 'pointer',
                        border: color === c ? '3px solid white' : '3px solid transparent',
                        transition: 'border 0.15s',
                      }}
                    />
                  ))}
                </div>
              </div>

              <IonButton
                expand="block"
                onClick={handleSave}
                disabled={!name.trim() || createMutation.isPending || updateMutation.isPending}
              >
                {createMutation.isPending || updateMutation.isPending ? <IonSpinner name="crescent" /> : 'Save'}
              </IonButton>
            </div>
          </IonContent>
        </IonModal>
      </IonContent>
    </IonPage>
  );
};

export default PeopleManagement;
