import React from 'react';
import {
  IonPage, IonContent, IonHeader, IonToolbar, IonTitle,
  IonButton, IonBackButton, IonButtons,
} from '@ionic/react';
import { useHistory } from 'react-router-dom';
import { usePeople, useCreateActivity } from '../api/hooks';
import LocalActivityEditor from '../components/activities/LocalActivityEditor';
import type { CreateActivityRequest } from '@family-center/contracts';

const CreateActivity: React.FC = () => {
  const history = useHistory();
  const { data: people = [] } = usePeople();
  const createMutation = useCreateActivity();

  const handleSave = async (data: CreateActivityRequest) => {
    await createMutation.mutateAsync(data);
    history.goBack();
  };

  return (
    <IonPage>
      <IonHeader>
        <IonToolbar>
          <IonButtons slot="start">
            <IonBackButton defaultHref="/week" />
          </IonButtons>
          <IonTitle>New Activity</IonTitle>
        </IonToolbar>
      </IonHeader>
      <IonContent>
        <LocalActivityEditor
          people={people}
          onSave={handleSave}
          isSaving={createMutation.isPending}
        />
      </IonContent>
    </IonPage>
  );
};

export default CreateActivity;
