cmake_minimum_required(VERSION 3.21)

project(git-checkout2 C)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_C_FLAGS "-O3")
set(CMAKE_INSTALL_PREFIX /usr/local)

add_executable(git-checkout2 main.c)
# set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fstack-usage")

if(NOT WIN32)
  string(ASCII 27 Esc)
  set(ColourReset "${Esc}[m")
  set(Yellow      "${Esc}[33m")
endif()

if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    target_compile_definitions(git-checkout2 PUBLIC DEBUG)
    string(ASCII 27 Esc)
    message(STATUS "${Yellow}Building git-checkout2 in debug mode.${ColourReset}")
endif()

set_property(TARGET git-checkout2 PROPERTY C_STANDARD 11)

file(CREATE_LINK ${CMAKE_BINARY_DIR}/compile_commands.json compile_commands.json SYMBOLIC)

install(TARGETS git-checkout2)
