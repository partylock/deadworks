#include "BuildGameSessionManifest.hpp"
#include "../Deadworks.hpp"
#include "../NativeHero.hpp"

__int64 __fastcall deadworks::hooks::Hook_BuildGameSessionManifest(void *thisptr, void **a2) {
    auto result = hooks::g_BuildGameSessionManifest.call<__int64>(thisptr, a2);

    if (!IsHeroPrecacheResolved())
        ResolveHeroPrecacheFns();

    void *manifest = *a2;
    PluginResourceCtx resourceCtx{};
    resourceCtx.manifest = manifest;
    // manifestSlot lets core defer or read fresh *a2 on flush without re-entering this hook.
    g_Deadworks.OnBuildGameSessionManifest(thisptr, a2, &resourceCtx);
    return result;
}
