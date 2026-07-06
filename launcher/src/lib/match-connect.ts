import type { MatchReadyPayload } from "./types";

const LOOPBACK_HOSTS = new Set(["127.0.0.1", "localhost", "host.docker.internal"]);

/** Address for steam://connect — loopback hosts become 127.0.0.1 for local dev. */
export function resolveMatchConnectAddr(match: MatchReadyPayload): string {
  const host = match.host.trim();
  const normalized = host.toLowerCase();
  const connectHost = LOOPBACK_HOSTS.has(normalized)
    ? (match.localHost?.trim() || "127.0.0.1")
    : host;
  return `${connectHost}:${match.port}`;
}
