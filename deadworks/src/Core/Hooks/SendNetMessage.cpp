#include "SendNetMessage.hpp"

#include "../Deadworks.hpp"

#include <networksystem/inetworkserializer.h>
#include <networksystem/netmessage.h>

namespace deadworks {
namespace hooks {

bool __fastcall Hook_CServerSideClient_SendNetMessage(CServerSideClientBase *thisptr, const CNetMessage *pData, unsigned char bufType) {
    if (g_Deadworks.OnPre_SendNetMessage(thisptr, pData))
        return true; // blocked — pretend success

    return g_CServerSideClient_SendNetMessage.thiscall<bool>(thisptr, pData, bufType);
}

} // namespace hooks
} // namespace deadworks
