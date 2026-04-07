#include "EntityIO.hpp"

#include "../Deadworks.hpp"

namespace deadworks {
namespace hooks {

void __fastcall Hook_CEntityInstance_AcceptInput(CEntityInstance *thisptr, const char *inputName,
                                                  void *activator, void *caller, const char *value) {
    // Forward to managed before calling original
    g_Deadworks.OnEntityAcceptInput(thisptr, activator, caller, inputName, value);

    g_CEntityInstance_AcceptInput.thiscall<void>(thisptr, inputName, activator, caller, value);
}

void __fastcall Hook_CEntityInstance_FireOutput(void *thisptr, void *outputData,
                                                 void *activator, void *caller, void *variant, float delay) {
    // FireOutput is more complex - the output name is embedded in the outputData structure.
    // For now, forward the call and let it pass through. Once the output name extraction
    // is known, we can fire the managed callback.
    // TODO: Extract output name from outputData and call g_Deadworks.OnEntityFireOutput()

    g_CEntityInstance_FireOutput.thiscall<void>(thisptr, outputData, activator, caller, variant, delay);
}

} // namespace hooks
} // namespace deadworks
