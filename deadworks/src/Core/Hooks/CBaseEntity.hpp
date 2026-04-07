#pragma once

#include <safetyhook.hpp>

class CBaseEntity;
class CTakeDamageResult;
class CTakeDamageInfo;

namespace deadworks {
namespace hooks {

inline safetyhook::InlineHook g_CBaseEntity_TakeDamageOld;
void __fastcall Hook_CBaseEntity_TakeDamageOld(CBaseEntity *thisptr, CTakeDamageInfo *info, CTakeDamageResult *result);

// Touch hooks - virtual indices loaded from deadworks_mem.jsonc "virtuals" section.
// Resolved lazily from the first entity's vtable in OnEntityCreated.
inline safetyhook::InlineHook g_CBaseEntity_StartTouch;
void __fastcall Hook_CBaseEntity_StartTouch(CBaseEntity *thisptr, CBaseEntity *other);

inline safetyhook::InlineHook g_CBaseEntity_EndTouch;
void __fastcall Hook_CBaseEntity_EndTouch(CBaseEntity *thisptr, CBaseEntity *other);

} // namespace hooks
} // namespace deadworks
