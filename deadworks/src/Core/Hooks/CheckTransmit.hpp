#pragma once

#include <cstdint>

#include <eiface.h>

namespace deadworks {
namespace hooks {

using CheckTransmitFn = void (__fastcall *)(void *thisptr, CCheckTransmitInfo **ppInfoList, int nInfoCount,
                                             CBitVec<16384> &unionTransmitEdicts, CBitVec<16384> &unk,
                                             const Entity2Networkable_t **pNetworkables,
                                             const uint16 *pEntityIndicies, int nEntities);

inline CheckTransmitFn g_Source2GameEntities_CheckTransmit_Original = nullptr;

void __fastcall Hook_CheckTransmit(void *thisptr, CCheckTransmitInfo **ppInfoList, int nInfoCount,
                                    CBitVec<16384> &unionTransmitEdicts, CBitVec<16384> &unk,
                                    const Entity2Networkable_t **pNetworkables, const uint16 *pEntityIndicies,
                                    int nEntities);

} // namespace hooks
} // namespace deadworks
