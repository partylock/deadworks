#pragma once

#include <safetyhook.hpp>

class CServerSideClientBase;

namespace deadworks {
namespace hooks {

inline safetyhook::InlineHook g_ReplyConnection;
void __fastcall Hook_ReplyConnection(void *server, CServerSideClientBase *client);

} // namespace hooks
} // namespace deadworks
