#include "CheckTransmit.hpp"

#include "../Deadworks.hpp"

namespace deadworks {
namespace hooks {

void __fastcall Hook_CheckTransmit(void *thisptr, CCheckTransmitInfo **ppInfoList, int nInfoCount,
                                    CBitVec<16384> &unionTransmitEdicts, CBitVec<16384> &unk,
                                    const Entity2Networkable_t **pNetworkables, const uint16 *pEntityIndicies,
                                    int nEntities) {
    // Call original first - normal indirect call, compiler handles home space / stack frame correctly
    g_Source2GameEntities_CheckTransmit_Original(thisptr, ppInfoList, nInfoCount,
                                                  unionTransmitEdicts, unk,
                                                  pNetworkables, pEntityIndicies,
                                                  nEntities);

    // Dispatch to managed code - one call per player entry
    g_Deadworks.OnPost_CheckTransmit(ppInfoList, nInfoCount);
}

} // namespace hooks
} // namespace deadworks
