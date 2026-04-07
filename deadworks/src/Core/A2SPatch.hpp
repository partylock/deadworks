#pragma once

namespace deadworks {

// Applies an in-memory patch to engine2.dll that enables A2S_INFO query
// responses on community dedicated servers.
//
// NOPs the IsOfficialServer() and GMS/Advertise conditional jumps that block
// ISteamGameServer::SetAdvertiseServerActive(true). Once active, the server
// responds to A2S queries on the game port (shared query port mode).
namespace A2SPatch {

// Apply the patch. Returns true on success.
bool Apply();

} // namespace A2SPatch
} // namespace deadworks
