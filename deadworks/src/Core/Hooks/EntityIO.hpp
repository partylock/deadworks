#pragma once

#include <safetyhook.hpp>

class CEntityInstance;
class CEntityIdentity;

namespace deadworks {
namespace hooks {

// CEntityInstance::AcceptInput hook - intercepts all entity input calls
inline safetyhook::InlineHook g_CEntityInstance_AcceptInput;
void __fastcall Hook_CEntityInstance_AcceptInput(CEntityInstance *thisptr, const char *inputName,
                                                  void *activator, void *caller, const char *value);

// CEntityInstance::FireOutput hook - intercepts all entity output fires
// NOTE: Signature must be added to deadworks_mem.jsonc as "CEntityInstance::FireOutput"
inline safetyhook::InlineHook g_CEntityInstance_FireOutput;
void __fastcall Hook_CEntityInstance_FireOutput(void *thisptr, void *outputData,
                                                 void *activator, void *caller, void *variant, float delay);

} // namespace hooks
} // namespace deadworks
