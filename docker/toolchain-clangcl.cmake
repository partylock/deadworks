set(CMAKE_SYSTEM_NAME Windows)
set(CMAKE_SYSTEM_PROCESSOR AMD64)

set(CMAKE_C_COMPILER /usr/local/bin/clang-cl)
set(CMAKE_CXX_COMPILER /usr/local/bin/clang-cl)
set(CMAKE_LINKER /usr/local/bin/lld-link)
set(CMAKE_AR /usr/bin/llvm-lib-20)
set(CMAKE_RC_COMPILER /usr/bin/llvm-rc-20)

set(XWIN_DIR "/xwin")

set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded")
cmake_policy(SET CMP0091 NEW)

set(CMAKE_TRY_COMPILE_CONFIGURATION Release)

string(JOIN " " _COMMON_FLAGS
 "/EHsc"
 "/MT"
 "-Wno-unused-command-line-argument"
 "-fuse-ld=lld"
 "/imsvc${XWIN_DIR}/crt/include"
 "/imsvc${XWIN_DIR}/sdk/include/ucrt"
 "/imsvc${XWIN_DIR}/sdk/include/um"
 "/imsvc${XWIN_DIR}/sdk/include/shared"
)

set(CMAKE_C_FLAGS_INIT "${_COMMON_FLAGS}")
set(CMAKE_CXX_FLAGS_INIT "${_COMMON_FLAGS}")

set(CMAKE_C_FLAGS_DEBUG_INIT "/MT /Od")
set(CMAKE_CXX_FLAGS_DEBUG_INIT "/MT /Od")
set(CMAKE_C_FLAGS_RELEASE_INIT "/MT /O2 /DNDEBUG")
set(CMAKE_CXX_FLAGS_RELEASE_INIT "/MT /O2 /DNDEBUG")

string(JOIN " " _LINK_FLAGS
 "/libpath:${XWIN_DIR}/crt/lib/x86_64"
 "/libpath:${XWIN_DIR}/sdk/lib/um/x86_64"
 "/libpath:${XWIN_DIR}/sdk/lib/ucrt/x86_64"
)
set(CMAKE_EXE_LINKER_FLAGS_INIT "${_LINK_FLAGS}")
set(CMAKE_SHARED_LINKER_FLAGS_INIT "${_LINK_FLAGS}")
set(CMAKE_STATIC_LINKER_FLAGS_INIT "")

set(CMAKE_MT "CMAKE_MT-NOTFOUND")
