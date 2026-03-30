cmake_minimum_required(VERSION 3.20)

project(cjlib_cpp LANGUAGES CXX)

include(CTest)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

set(CJLIB_ROOT_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../.." CACHE PATH "Repository root")
set(CJLIB_FFI_CORE_INCLUDE_DIR "${CJLIB_ROOT_DIR}/ffi/core/include" CACHE PATH "Generated cjlib C ABI headers")
set(CJLIB_FFI_CORE_SHARED_LIB "${CJLIB_ROOT_DIR}/target/debug/libcjlib_ffi_core.so" CACHE FILEPATH "Built cjlib ffi shared library")

if(NOT EXISTS "${CJLIB_FFI_CORE_INCLUDE_DIR}/cjlib/cjlib.h")
  message(FATAL_ERROR "Missing generated C ABI header at ${CJLIB_FFI_CORE_INCLUDE_DIR}/cjlib/cjlib.h. Run `just ffi-header` first.")
endif()

if(NOT EXISTS "${CJLIB_FFI_CORE_SHARED_LIB}")
  message(FATAL_ERROR "Missing built cjlib ffi shared library at ${CJLIB_FFI_CORE_SHARED_LIB}. Build `cjlib-ffi-core` first.")
endif()

add_library(cjlib_ffi_core SHARED IMPORTED GLOBAL)
set_target_properties(
  cjlib_ffi_core
  PROPERTIES
    IMPORTED_LOCATION "${CJLIB_FFI_CORE_SHARED_LIB}"
)

add_library(cjlib_cpp INTERFACE)
target_include_directories(
  cjlib_cpp
  INTERFACE
    "${CMAKE_CURRENT_SOURCE_DIR}/include"
    "${CJLIB_FFI_CORE_INCLUDE_DIR}"
)
target_link_libraries(cjlib_cpp INTERFACE cjlib_ffi_core)

if(BUILD_TESTING)
  add_executable(cjlib_cpp_smoke tests/smoke.cpp)
  target_link_libraries(cjlib_cpp_smoke PRIVATE cjlib_cpp)
  target_compile_definitions(
    cjlib_cpp_smoke
    PRIVATE
      CJLIB_FIXTURE_PATH="${CJLIB_ROOT_DIR}/tests/data/v2_0/minimal.city.json"
  )
  get_filename_component(CJLIB_FFI_CORE_LIB_DIR "${CJLIB_FFI_CORE_SHARED_LIB}" DIRECTORY)
  set_target_properties(cjlib_cpp_smoke PROPERTIES BUILD_RPATH "${CJLIB_FFI_CORE_LIB_DIR}")
  add_test(NAME cjlib_cpp_smoke COMMAND cjlib_cpp_smoke)
endif()
