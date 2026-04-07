#include "ReplyConnection.hpp"

#include "../Deadworks.hpp"

namespace deadworks {
namespace hooks {

void __fastcall Hook_ReplyConnection(void *server, CServerSideClientBase *client) {
    g_Deadworks.OnPre_ReplyConnection(server, client);
    g_ReplyConnection.call<void>(server, client);
    g_Deadworks.OnPost_ReplyConnection(server, client);
}

} // namespace hooks
} // namespace deadworks
