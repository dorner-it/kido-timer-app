import type { ChannelStatus, DeviceMode, StateFlag } from "./types";

/** Central German UI strings. All visible text in the app comes from here. */
export const t = {
  app: {
    brand: "TRV·Kocab",
    subtitle: "Sportzeit-Konsole",
    title: "TRV Kocab Sportzeitmessung",
  },

  topbar: {
    menu: "Menü öffnen",
    closeMenu: "Menü schließen",
    localTime: "Ortszeit",
    fps: "Bilder/s",
    frames: "Bilder",
    mode: "Modus",
    lane: "Bahn",
    state: "Status",
  },

  conn: {
    statusIdle: "Bereit",
    statusConnecting: "Verbinde…",
    statusConnected: "Live",
    statusError: "Störung",
    noSource: "Keine Quelle",
  },

  setup: {
    welcomeTitle: "Willkommen",
    welcomeBody:
      "Wähle den Anschluss zu deiner TRV Kocab Sportzeituhr oder starte den Demo-Modus mit einer aufgezeichneten Datei.",
    tabSerial: "Sportzeituhr",
    tabDemo: "Demo-Wiedergabe",
    serialPort: "Anschluss",
    baudRate: "Baudrate",
    rescan: "Neu suchen",
    scanning: "Suche …",
    noPorts: "Kein Anschluss erkannt",
    rememberLabel: "Beim nächsten Start automatisch verbinden",
    saveAndConnect: "Speichern & Verbinden",
    connect: "Verbinden",
    pickFile: "Aufnahme-Datei auswählen (.txt)",
    speed: "Geschwindigkeit",
    speedMax: "max",
    startDemo: "Demo starten",
    cancel: "Abbrechen",
    settings: "Einstellungen",
  },

  drawer: {
    title: "Steuerung",
    sectionConnection: "Verbindung",
    sectionDevice: "Gerät",
    sectionView: "Ansicht",
    sectionAppearance: "Darstellung",
    themeLabel: "Erscheinungsbild",
    themeLight: "Hell",
    themeDark: "Dunkel",
    source: "Quelle",
    notConnected: "Nicht verbunden",
    edit: "Verbindung ändern",
    disconnect: "Trennen",
    reset: "Gerät zurücksetzen",
    resetConfirm: "Wirklich alle Bahnen am Gerät zurücksetzen?",
    resetConfirmAction: "Zurücksetzen bestätigen",
    resetCommand: "Sendet RST\\r über die serielle Verbindung.",
    openProtocol: "Lauf-Protokoll öffnen",
    backToLanes: "Zurück zur Bahn-Ansicht",
  },

  lanes: {
    sectionTitle: "Bahnen",
    sectionKicker: "Live",
    laneShort: "BAHN",
    correction: "Korrektur",
    original: "Original",
    penalty: "Strafe",
    total: "Gesamt",
    clearCorrection: "Korrektur zurücksetzen",
    minus5: "−5 s",
    minus1: "−1 s",
    plus1: "+1 s",
    plus5: "+5 s",
    inactivePlaceholder: "––.–––",
    focusHint:
      "Bahn anklicken (oder Tasten 1–4) · +/− verstellt um ±5 s · C löscht Korrektur",
  },

  protocol: {
    title: "Lauf-Protokoll",
    subtitle: "Bestätigte Zeiten · neueste zuerst",
    empty: "Noch keine bestätigten Zeiten",
    kpiCount: "Einträge",
    kpiFastest: "Schnellste",
    kpiSlowest: "Langsamste",
    kpiCorrected: "Mit Korrektur",
    columnIndex: "Lauf",
    columnLane: "Bahn",
    columnOriginal: "Original-Zeit",
    columnPenalty: "Strafzeit",
    columnTotal: "Gesamt-Zeit",
    columnTimestamp: "Zeitstempel",
    columnDate: "Datum",
    exportCsv: "Als CSV exportieren",
    clearAll: "Protokoll leeren",
    clearConfirm: "Alle Einträge im Protokoll löschen?",
    clearConfirmAction: "Leeren bestätigen",
    saved: "Gespeichert: ",
    exportFailed: "Export fehlgeschlagen",
  },

  errors: {
    title: "Fehler vom Gerät",
    dismiss: "Verbergen",
    notConnectedAction: "Nicht verbunden",
    needDesktopApp: "Diese Aktion benötigt die Desktop-App",
  },

  updates: {
    available: "Update verfügbar",
    versionPrefix: "Version",
    installNow: "Jetzt installieren",
    later: "Später",
    downloading: "Wird heruntergeladen…",
    installing: "Installation läuft, App startet neu…",
    failed: "Update fehlgeschlagen",
    checkNow: "Auf Updates prüfen",
    checking: "Suche nach Updates…",
    upToDate: "App ist auf dem neuesten Stand",
  },

  csv: {
    headers: [
      "Lauf",
      "Bahn",
      "Original-Zeit (s,ms)",
      "Strafzeit (s,ms)",
      "Gesamt-Zeit (s,ms)",
      "Zeitstempel",
      "Datum",
    ],
  },
};

const STATUS: Record<ChannelStatus, string> = {
  inactive: "Inaktiv",
  running: "Läuft",
  captured: "Erfasst",
  confirmed: "Bestätigt",
  unknown: "Unbekannt",
};

export function statusLabelDE(s: ChannelStatus): string {
  return STATUS[s];
}

const DEVICE: Record<DeviceMode, string> = {
  standby: "Bereit",
  active: "Aktiv",
  unknown: "Unbekannt",
};

export function deviceModeDE(d: DeviceMode): string {
  return DEVICE[d];
}

const STATE_FLAG: Record<StateFlag, string> = {
  measuring: "Misst",
  armed: "Scharf",
  unknown: "Unbekannt",
};

export function stateFlagDE(s: StateFlag): string {
  return STATE_FLAG[s];
}
