export function getApiBaseUrl(apiEndpoint: string): string {
  if (import.meta.env.DEV && apiEndpoint === "local") {
    return "http://localhost:3001/api/v1";
  }
  return import.meta.env.VITE_PARTYLOCK_API_URL || "http://localhost:3001/api/v1";
}

export function getWsBaseUrl(apiEndpoint: string): string {
  const apiBase = getApiBaseUrl(apiEndpoint);
  return apiBase.replace(/\/api\/v1\/?$/, "");
}
