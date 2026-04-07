#include "A2SPatch.hpp"
#include "Deadworks.hpp"
#include "../Lib/Module.hpp"
#include "../Memory/Scanner.hpp"

#include <safetyhook.hpp>

namespace deadworks {
namespace A2SPatch {

// SetAdvertiseServerActive gate
//   call IsOfficialServer / test al,al / jz / ... / test eax,eax / jle
// Two conditional jumps block the call on community servers.
static constexpr auto kSigAdvertise =
    "E8 ?? ?? ?? ?? 84 C0 74 ?? 48 8B 0D ?? ?? ?? ?? ?? ?? ?? FF 90 ?? ?? ?? ?? 45 33 C0";
static constexpr ptrdiff_t kJzOffset  = 7;   // jz  after IsOfficialServer check
static constexpr ptrdiff_t kJleOffset = 46;  // jle after GMS/Advertise GetInt check

static constexpr uint8_t kNop = 0x90;

static bool PatchBytes(uint8_t *addr, const uint8_t *bytes, size_t len) {
    auto unprotect = safetyhook::unprotect(addr, len);
    if (!unprotect) return false;
    std::memcpy(addr, bytes, len);
    return true;
}

bool Apply() {
    Module engine2("engine2.dll");
    if (!engine2.IsValid()) {
        g_Log->Error("[A2S] Failed to get engine2.dll module");
        return false;
    }

    auto textSection = engine2.GetSectionMemory(".text");

    auto sig = Scanner::ParseSignature(kSigAdvertise);
    if (!sig) {
        g_Log->Error("[A2S] Failed to parse advertise signature: {}", sig.error());
        return false;
    }

    auto match = Scanner::FindFirst(textSection, *sig);
    if (!match) {
        g_Log->Error("[A2S] Advertise gate signature not found in engine2.dll");
        return false;
    }

    auto *base = reinterpret_cast<uint8_t *>(*match);
    uint8_t nops[2] = {kNop, kNop};
    bool allOk = true;

    // NOP the jz (IsOfficialServer gate)
    auto *jzAddr = base + kJzOffset;
    if (jzAddr[0] == kNop && jzAddr[1] == kNop) {
        g_Log->Info("[A2S] Advertise jz already NOPed");
    } else {
        g_Log->Info("[A2S] Advertise jz: {:02X} {:02X} -> 90 90", jzAddr[0], jzAddr[1]);
        if (!PatchBytes(jzAddr, nops, 2)) {
            g_Log->Error("[A2S] Failed to NOP advertise jz (VirtualProtect failed)");
            allOk = false;
        }
    }

    // NOP the jle (GMS/Advertise GetInt gate)
    auto *jleAddr = base + kJleOffset;
    if (jleAddr[0] == kNop && jleAddr[1] == kNop) {
        g_Log->Info("[A2S] Advertise jle already NOPed");
    } else {
        g_Log->Info("[A2S] Advertise jle: {:02X} {:02X} -> 90 90", jleAddr[0], jleAddr[1]);
        if (!PatchBytes(jleAddr, nops, 2)) {
            g_Log->Error("[A2S] Failed to NOP advertise jle (VirtualProtect failed)");
            allOk = false;
        }
    }

    if (allOk)
        g_Log->Info("[A2S] Advertise gate patched — A2S_INFO enabled on game port");

    return allOk;
}

} // namespace A2SPatch
} // namespace deadworks
