export interface ModInfo {
  name: string;
  type: string;
  version: string;
}

export interface PlayerInfo {
  name: string;
  hero: string;
  team: number;
  kills: number;
  deaths: number;
  assists: number;
  level: number;
}

export interface MatchReadyPayload {
  matchId: string;
  draftRoomId: string;
  host: string;
  port: number;
  connectCommand: string;
  localHost?: string;
  localConnectCommand?: string;
}

export interface MatchProvisionStep {
  step: string;
  label: string;
  progress?: number;
  total?: number;
}

export interface MatchProvisioningPayload {
  matchId: string;
  draftRoomId: string;
  label: string;
  steps: MatchProvisionStep[];
}

export interface MatchConnectStatusPayload {
  matchId: string;
  draftRoomId: string;
  connectedCount: number;
  rosterCount: number;
  deadlineAt: string;
  remainingSeconds?: number;
}

export interface MatchFailedNotification {
  matchId: string;
  draftRoomId: string;
  reason: string;
}

export type ContentKind = "map" | "addon";

export interface ContentManifestItem {
  filename: string;
  kind: ContentKind;
  version: number;
  compressed_size: number;
  download_url: string;
}

export interface Server {
  id: string;
  name: string;
  address: string;
  raw_address: string;
  country: string;
  online: boolean;
  player_count: number;
  max_players: number;
  map: string;
  players: PlayerInfo[];
  mods: ModInfo[];
  content_addons: string[];
  extra_maps: string[];
  content?: ContentManifestItem[];
  last_heartbeat: string | null;
}

export interface DownloadProgress {
  name: string;
  status: string;
  bytes_downloaded: number;
  total_bytes: number;
  item_index: number;
  total_items: number;
}

export interface ConnectResult {
  success: boolean;
  method: string;
  message: string;
}
