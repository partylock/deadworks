#pragma once

#include <safetyhook.hpp>

class CNetMessage;
class CServerSideClientBase;
class INetworkChannelNotify;
class INetworkMessageProcessingPreFilter;

namespace deadworks {
namespace hooks {

inline safetyhook::InlineHook g_CServerSideClientBase_FilterMessage;
bool __fastcall Hook_CServerSideClientBase_FilterMessage(INetworkMessageProcessingPreFilter *thisptr, const CNetMessage *pData);

inline safetyhook::InlineHook g_CServerSideClientBase_IsReservedSlot;
bool __fastcall Hook_CServerSideClientBase_IsReservedSlot(CServerSideClientBase *thisptr);

} // namespace hooks
} // namespace deadworks
