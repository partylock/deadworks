#pragma once

#include <safetyhook.hpp>

class CNetMessage;
class CServerSideClientBase;

namespace deadworks {
namespace hooks {

inline safetyhook::InlineHook g_CServerSideClient_SendNetMessage;
bool __fastcall Hook_CServerSideClient_SendNetMessage(CServerSideClientBase *thisptr, const CNetMessage *pData, unsigned char bufType);

} // namespace hooks
} // namespace deadworks
