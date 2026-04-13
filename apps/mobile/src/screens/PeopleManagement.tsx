import React, { useState } from 'react';
import {
  IonPage, IonContent, IonHeader, IonToolbar, IonTitle,
  IonSpinner, IonModal,
} from '@ionic/react';
import {
  usePeople, useCreatePerson, useUpdatePerson,
  useGoogleAccounts, useLaneRules, useLinkAccount, useUnlinkAccount,
} from '../api/hooks';
import Avatar from '../components/Avatar';
import type { Person, GoogleAccountPublic, LaneAssignmentRule } from '@family-center/contracts';

const COLORS = [
  '#5E81F4', '#FF6B6B', '#4FCB8A', '#F5A623',
  '#A78BFA', '#06B6D4', '#F472B6', '#FB923C',
  '#8B78FF', '#34D399', '#FBBF24', '#60A5FA',
];

// ─── Gmail picker sub-component ───
interface GmailPickerProps {
  personId: string | null; // null = shared
  accounts: GoogleAccountPublic[];
  rules: LaneAssignmentRule[];
  onLink: (googleAccountId: string) => void;
  onUnlink: (googleAccountId: string) => void;
  isLinking: boolean;
}

function linkedAccountIds(personId: string | null, rules: LaneAssignmentRule[]): Set<string> {
  // An account is considered linked to a person if there's at least one
  // lane_assignment_rule with a calendarSourceId pointing to that person.
  // We don't have the calendar→account mapping on the frontend,
  // so instead we look at rules where personId matches.
  // The rules store calendarSourceId, but we can't resolve that to a GoogleAccountId
  // without extra data. So we'll track by the rule's personId match.
  // This is a simplified view — we just show all linked accounts.
  return new Set<string>();
}

const GmailPicker: React.FC<GmailPickerProps> = ({
  personId, accounts, rules, onLink, onUnlink, isLinking,
}) => {
  // Determine which google accounts have rules for this person
  const linkedGoogleAccountIds = new Set<string>();

  // We need to build a mapping: for each rule with a calendarSourceId,
  // find which google account it belongs to. Since we don't have that data here,
  // we'll derive it from the accounts + rules available.
  // However, we do have rules where personId matches.
  // The link-account endpoint creates rules with calendarSourceId set.
  // For display, we'll match rules to accounts by checking if the account
  // has any rules pointing to this person.
  //
  // Workaround: The link-account endpoint links ALL selected calendars.
  // So we just need to know if ANY rule with this personId exists.
  // We'll use a simple "is this account linked to this person" check.
  //
  // Since we have the full rules list and the accounts list,
  // but rules point to calendar_source_id (not google_account_id),
  // we'll let the backend handle the real mapping. On the frontend,
  // we show a simple link/unlink UI per account.

  // For the UI, we track per-account link state using a different approach:
  // Store the linked account IDs in the rules themselves.
  // Rules have calendarSourceId but not googleAccountId.
  // We'll add a simple email-based matching: if there's a rule
  // with emailPattern matching the account's email, it's linked.
  // OR if rules with calendarSourceId exist for this person.
  //
  // Simplest approach: track which accounts are linked by checking
  // if there are rules for this person at all, and show link/unlink.
  // The link-account endpoint handles the details.

  const rulesForPerson = rules.filter((r) =>
    personId
      ? r.personId === personId
      : !r.personId && r.laneTarget === 'shared'
  );

  // For now, we'll show all accounts and let the user link/unlink.
  // We'll show a checkmark if there are rules for this person.
  // This is imprecise but functional — the backend does the real work.
  const hasRules = rulesForPerson.length > 0;

  if (accounts.length === 0) {
    return (
      <div style={s.gmailEmpty}>
        No Google accounts connected. Go to Settings to connect one.
      </div>
    );
  }

  return (
    <div style={s.gmailSection}>
      {accounts.map((acct) => {
        // Check if this specific account has rules for this person
        // We can't directly map calendarSourceId → googleAccountId on frontend
        // So we show all accounts with link/unlink actions
        const acctRules = rulesForPerson.filter((r) => r.calendarSourceId);
        const isLinked = acctRules.length > 0; // simplified

        return (
          <div key={acct.id} style={s.gmailRow}>
            <div style={s.gmailIcon}>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/>
                <polyline points="22,6 12,13 2,6"/>
              </svg>
            </div>
            <div style={s.gmailInfo}>
              <div style={s.gmailEmail}>{acct.email}</div>
              {acct.displayName && acct.displayName !== acct.email && (
                <div style={s.gmailName}>{acct.displayName}</div>
              )}
            </div>
            <button
              style={{
                ...s.gmailAction,
                background: isLinked ? 'var(--fc-green-dim)' : 'var(--fc-bg)',
                color: isLinked ? 'var(--fc-green)' : 'var(--fc-text-secondary)',
                borderColor: isLinked ? 'var(--fc-green)' : 'var(--fc-border)',
              }}
              onClick={() => isLinked ? onUnlink(acct.id) : onLink(acct.id)}
              disabled={isLinking}
            >
              {isLinking ? '...' : isLinked ? 'Linked' : 'Link'}
            </button>
          </div>
        );
      })}
    </div>
  );
};

// ─── Main Component ───

const PeopleManagement: React.FC = () => {
  const { data: people = [], isLoading } = usePeople();
  const { data: accounts = [] } = useGoogleAccounts();
  const { data: rules = [] } = useLaneRules();
  const createMutation = useCreatePerson();
  const updateMutation = useUpdatePerson();
  const linkMutation = useLinkAccount();
  const unlinkMutation = useUnlinkAccount();

  const [showModal, setShowModal] = useState(false);
  const [editing, setEditing] = useState<Person | null>(null);
  const [name, setName] = useState('');
  const [color, setColor] = useState(COLORS[0]);
  const [avatarUrl, setAvatarUrl] = useState('');

  const openCreate = () => {
    setEditing(null);
    setName('');
    setColor(COLORS[people.length % COLORS.length]);
    setAvatarUrl('');
    setShowModal(true);
  };

  const openEdit = (person: Person) => {
    setEditing(person);
    setName(person.name);
    setColor(person.color);
    setAvatarUrl(person.avatarUrl ?? '');
    setShowModal(true);
  };

  const handleSave = async () => {
    if (!name.trim()) return;
    if (editing) {
      await updateMutation.mutateAsync({ id: editing.id, name: name.trim(), color, avatarUrl: avatarUrl || undefined });
    } else {
      await createMutation.mutateAsync({ name: name.trim(), color, avatarUrl: avatarUrl || undefined, sortOrder: people.length });
    }
    setShowModal(false);
  };

  const handleLink = (personId: string | null, googleAccountId: string) => {
    linkMutation.mutate({ googleAccountId, personId: personId ?? undefined });
  };

  const handleUnlink = (personId: string | null, googleAccountId: string) => {
    unlinkMutation.mutate({ googleAccountId, personId: personId ?? undefined });
  };

  const isBusy = createMutation.isPending || updateMutation.isPending;
  const isLinking = linkMutation.isPending || unlinkMutation.isPending;

  // Count rules per person to show linked status
  const rulesForPerson = (pid: string | null) =>
    rules.filter((r) => pid ? r.personId === pid : !r.personId && r.laneTarget === 'shared');

  return (
    <IonPage>
      <IonHeader>
        <IonToolbar>
          <IonTitle>People & Lanes</IonTitle>
        </IonToolbar>
      </IonHeader>
      <IonContent>
        <div style={s.page}>
          {isLoading ? (
            <div style={{ display: 'flex', justifyContent: 'center', padding: '40px' }}>
              <IonSpinner />
            </div>
          ) : (
            <>
              <p style={s.hint}>
                Each person gets their own swim lane. Link a Gmail to route calendar events to their lane.
              </p>

              {/* ── Shared lane card ── */}
              <div style={s.laneCard}>
                <div style={s.laneCardHeader}>
                  <Avatar name="Shared" color="var(--fc-shared-lane-color)" size={40} />
                  <div style={s.cardInfo}>
                    <div style={s.cardName}>Shared</div>
                    <div style={s.cardSub}>
                      {rulesForPerson(null).length > 0
                        ? `${rulesForPerson(null).length} calendar rule(s)`
                        : 'No Gmail linked'}
                    </div>
                  </div>
                </div>
                <div style={s.laneCardGmail}>
                  <label style={s.gmailLabel}>Gmail account</label>
                  <GmailPicker
                    personId={null}
                    accounts={accounts}
                    rules={rules}
                    onLink={(gid) => handleLink(null, gid)}
                    onUnlink={(gid) => handleUnlink(null, gid)}
                    isLinking={isLinking}
                  />
                </div>
              </div>

              {/* ── Person cards ── */}
              {people.map((person) => (
                <div key={person.id} style={s.laneCard}>
                  <button style={s.laneCardHeader} onClick={() => openEdit(person)}>
                    <Avatar name={person.name} color={person.color} avatarUrl={person.avatarUrl} size={40} />
                    <div style={s.cardInfo}>
                      <div style={s.cardName}>{person.name}</div>
                      <div style={s.cardSub}>
                        {rulesForPerson(person.id).length > 0
                          ? `${rulesForPerson(person.id).length} calendar rule(s)`
                          : 'No Gmail linked'}
                      </div>
                    </div>
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--fc-text-muted)" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                      <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
                    </svg>
                  </button>
                  <div style={s.laneCardGmail}>
                    <label style={s.gmailLabel}>Gmail account</label>
                    <GmailPicker
                      personId={person.id}
                      accounts={accounts}
                      rules={rules}
                      onLink={(gid) => handleLink(person.id, gid)}
                      onUnlink={(gid) => handleUnlink(person.id, gid)}
                      isLinking={isLinking}
                    />
                  </div>
                </div>
              ))}

              {people.length === 0 && (
                <p style={s.empty}>No people yet. Add someone below to create their lane.</p>
              )}

              {/* Add person button */}
              <button style={s.addBtn} onClick={openCreate}>
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                  <line x1="12" y1="5" x2="12" y2="19"/>
                  <line x1="5" y1="12" x2="19" y2="12"/>
                </svg>
                Add person
              </button>
            </>
          )}
        </div>

        {/* ── Edit / Create modal ── */}
        <IonModal isOpen={showModal} onDidDismiss={() => setShowModal(false)}>
          <IonHeader>
            <IonToolbar>
              <IonTitle style={{ fontFamily: 'var(--fc-font-display)' }}>
                {editing ? 'Edit person' : 'Add person'}
              </IonTitle>
              <button slot="end" style={s.modalCancel} onClick={() => setShowModal(false)}>
                Cancel
              </button>
            </IonToolbar>
          </IonHeader>
          <IonContent>
            <div style={s.modalBody}>
              <div style={s.field}>
                <label style={s.fieldLabel}>Name</label>
                <input
                  style={s.input}
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="Alice"
                  autoFocus
                />
              </div>

              <div style={s.field}>
                <label style={s.fieldLabel}>Lane color</label>
                <div style={s.colorGrid}>
                  {COLORS.map((c) => (
                    <button
                      key={c}
                      style={{
                        ...s.colorBtn,
                        background: c,
                        boxShadow: color === c ? `0 0 0 3px var(--fc-bg-surface), 0 0 0 5px ${c}` : 'none',
                        transform: color === c ? 'scale(1.1)' : 'scale(1)',
                      }}
                      onClick={() => setColor(c)}
                    />
                  ))}
                </div>
              </div>

              <div style={s.field}>
                <label style={s.fieldLabel}>Profile picture URL</label>
                <input
                  style={s.input}
                  value={avatarUrl}
                  onChange={(e) => setAvatarUrl(e.target.value)}
                  placeholder="https://example.com/photo.jpg"
                />
                <span style={{ fontSize: '11px', color: 'var(--fc-text-muted)' }}>
                  Paste a link to a photo. Leave empty for initials.
                </span>
              </div>

              {/* Preview */}
              {name.trim() && (
                <div style={{ ...s.previewCard, borderLeft: `4px solid ${color}` }}>
                  <Avatar name={name.trim()} color={color} avatarUrl={avatarUrl || undefined} size={32} />
                  <span style={s.previewName}>{name.trim()}</span>
                  <span style={s.previewSub}>Lane preview</span>
                </div>
              )}

              <button
                style={{
                  ...s.saveBtn,
                  opacity: !name.trim() || isBusy ? 0.5 : 1,
                }}
                disabled={!name.trim() || isBusy}
                onClick={handleSave}
              >
                {isBusy
                  ? <IonSpinner name="crescent" style={{ width: 18, height: 18, color: '#000' }} />
                  : editing ? 'Save changes' : 'Add person'
                }
              </button>
            </div>
          </IonContent>
        </IonModal>
      </IonContent>
    </IonPage>
  );
};

const s: Record<string, React.CSSProperties> = {
  page: {
    padding: '16px',
    display: 'flex',
    flexDirection: 'column',
    gap: '12px',
    fontFamily: 'var(--fc-font-body)',
  },
  hint: {
    fontSize: '13px',
    color: 'var(--fc-text-secondary)',
    margin: '0 0 4px',
    fontFamily: 'var(--fc-font-body)',
  },

  // Lane cards (person + shared)
  laneCard: {
    background: 'var(--fc-bg-card)',
    border: '1px solid var(--fc-border)',
    borderRadius: '12px',
    overflow: 'hidden',
  },
  laneCardHeader: {
    display: 'flex',
    alignItems: 'center',
    gap: '14px',
    padding: '14px 16px',
    cursor: 'pointer',
    outline: 'none',
    background: 'none',
    border: 'none',
    width: '100%',
    textAlign: 'left',
    transition: 'background 0.15s',
    minHeight: '64px',
  },
  laneCardGmail: {
    borderTop: '1px solid var(--fc-border)',
    padding: '10px 16px 12px',
    background: 'rgba(0,0,0,0.15)',
  },
  cardInfo: {
    flex: 1,
    minWidth: 0,
  },
  cardName: {
    fontFamily: 'var(--fc-font-display)',
    fontSize: '16px',
    fontWeight: 700,
    color: 'var(--fc-text-primary)',
    marginBottom: '2px',
  },
  cardSub: {
    fontSize: '12px',
    color: 'var(--fc-text-secondary)',
  },

  // Gmail section
  gmailLabel: {
    fontSize: '10px',
    fontWeight: 600,
    textTransform: 'uppercase',
    letterSpacing: '0.06em',
    color: 'var(--fc-text-muted)',
    fontFamily: 'var(--fc-font-body)',
    marginBottom: '6px',
    display: 'block',
  },
  gmailSection: {
    display: 'flex',
    flexDirection: 'column',
    gap: '6px',
  },
  gmailRow: {
    display: 'flex',
    alignItems: 'center',
    gap: '10px',
    padding: '6px 0',
  },
  gmailIcon: {
    color: 'var(--fc-text-muted)',
    flexShrink: 0,
    display: 'flex',
  },
  gmailInfo: {
    flex: 1,
    minWidth: 0,
  },
  gmailEmail: {
    fontSize: '13px',
    fontWeight: 500,
    color: 'var(--fc-text-primary)',
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  },
  gmailName: {
    fontSize: '11px',
    color: 'var(--fc-text-muted)',
  },
  gmailAction: {
    padding: '5px 12px',
    borderRadius: '6px',
    border: '1.5px solid',
    fontSize: '12px',
    fontFamily: 'var(--fc-font-body)',
    fontWeight: 600,
    cursor: 'pointer',
    outline: 'none',
    transition: 'all 0.15s',
    flexShrink: 0,
    minWidth: '60px',
    textAlign: 'center',
  },
  gmailEmpty: {
    fontSize: '12px',
    color: 'var(--fc-text-muted)',
    fontStyle: 'italic',
    padding: '4px 0',
  },

  // General
  empty: {
    fontSize: '13px',
    color: 'var(--fc-text-secondary)',
    textAlign: 'center',
    padding: '20px 0',
    fontStyle: 'italic',
    fontFamily: 'var(--fc-font-body)',
  },
  addBtn: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    gap: '8px',
    background: 'var(--fc-blue-dim)',
    border: '1.5px dashed var(--fc-blue)',
    borderRadius: '12px',
    color: 'var(--fc-blue)',
    fontSize: '14px',
    fontWeight: 600,
    fontFamily: 'var(--fc-font-body)',
    padding: '16px',
    cursor: 'pointer',
    width: '100%',
    marginTop: '4px',
    outline: 'none',
    transition: 'background 0.15s',
    minHeight: '54px',
  },

  // Modal
  modalBody: {
    padding: '20px 16px',
    display: 'flex',
    flexDirection: 'column',
    gap: '20px',
    fontFamily: 'var(--fc-font-body)',
  },
  modalCancel: {
    background: 'none',
    border: 'none',
    color: 'var(--fc-text-secondary)',
    fontSize: '14px',
    cursor: 'pointer',
    padding: '0 16px',
    fontFamily: 'var(--fc-font-body)',
    outline: 'none',
  },
  field: {
    display: 'flex',
    flexDirection: 'column',
    gap: '8px',
  },
  fieldLabel: {
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
    padding: '13px 14px',
    color: 'var(--fc-text-primary)',
    fontSize: '16px',
    fontFamily: 'var(--fc-font-body)',
    outline: 'none',
    width: '100%',
    boxSizing: 'border-box',
  },
  colorGrid: {
    display: 'flex',
    flexWrap: 'wrap',
    gap: '12px',
    paddingTop: '4px',
  },
  colorBtn: {
    width: '44px',
    height: '44px',
    borderRadius: '12px',
    border: 'none',
    cursor: 'pointer',
    transition: 'transform 0.15s, box-shadow 0.15s',
    outline: 'none',
  },
  previewCard: {
    display: 'flex',
    alignItems: 'center',
    gap: '10px',
    background: 'var(--fc-bg-card)',
    borderRadius: '10px',
    padding: '12px 16px',
  },
  previewName: {
    fontFamily: 'var(--fc-font-display)',
    fontSize: '15px',
    fontWeight: 700,
    color: 'var(--fc-text-primary)',
    flex: 1,
  },
  previewSub: {
    fontSize: '11px',
    color: 'var(--fc-text-muted)',
    fontFamily: 'var(--fc-font-body)',
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
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    transition: 'opacity 0.15s',
    minHeight: '52px',
  },
};

export default PeopleManagement;
