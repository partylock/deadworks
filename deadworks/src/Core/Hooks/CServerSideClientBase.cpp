#include "CServerSideClientBase.hpp"

#include "../Deadworks.hpp"

namespace deadworks {
namespace hooks {

bool __fastcall Hook_CServerSideClientBase_FilterMessage(INetworkMessageProcessingPreFilter *thisptr, const CNetMessage *pData) {
    auto result = g_Deadworks.OnPre_CServerSideClientBase_FilterMessage(thisptr, pData);
    if (result.has_value())
        return *result;

    return g_CServerSideClientBase_FilterMessage.thiscall<bool>(thisptr, pData);
}

bool __fastcall Hook_CServerSideClientBase_IsReservedSlot(CServerSideClientBase *thisptr) {
    return false;
}

} // namespace hooks
} // namespace deadworks
