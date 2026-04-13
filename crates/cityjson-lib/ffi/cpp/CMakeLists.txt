cmake_minimum_required(VERSION 3.20)

project(cityjson_lib_cpp LANGUAGES CXX)

include(CTest)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

set(CITYJSON_LIB_ROOT_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../.." CACHE PATH "Repository root")
set(CITYJSON_LIB_FFI_CORE_INCLUDE_DIR "${CITYJSON_LIB_ROOT_DIR}/ffi/core/include" CACHE PATH "Generated cityjson_lib C ABI headers")
set(CITYJSON_LIB_FFI_CORE_SHARED_LIB "${CITYJSON_LIB_ROOT_DIR}/target/debug/libcityjson_lib_ffi_core.so" CACHE FILEPATH "Built cityjson_lib ffi shared library")

if(NOT EXISTS "${CITYJSON_LIB_FFI_CORE_INCLUDE_DIR}/cityjson_lib/cityjson_lib.h")
  message(FATAL_ERROR "Missing generated C ABI header at ${CITYJSON_LIB_FFI_CORE_INCLUDE_DIR}/cityjson_lib/cityjson_lib.h. Run `just ffi-header` first.")
endif()

if(NOT EXISTS "${CITYJSON_LIB_FFI_CORE_SHARED_LIB}")
  message(FATAL_ERROR "Missing built cityjson_lib ffi shared library at ${CITYJSON_LIB_FFI_CORE_SHARED_LIB}. Build `cityjson-lib-ffi-core` first.")
endif()

add_library(cityjson_lib_ffi_core SHARED IMPORTED GLOBAL)
set_target_properties(
  cityjson_lib_ffi_core
  PROPERTIES
    IMPORTED_LOCATION "${CITYJSON_LIB_FFI_CORE_SHARED_LIB}"
)

add_library(cityjson_lib_cpp INTERFACE)
target_include_directories(
  cityjson_lib_cpp
  INTERFACE
    "${CMAKE_CURRENT_SOURCE_DIR}/include"
    "${CITYJSON_LIB_FFI_CORE_INCLUDE_DIR}"
)
target_link_libraries(cityjson_lib_cpp INTERFACE cityjson_lib_ffi_core)

if(BUILD_TESTING)
  add_executable(cityjson_lib_cpp_smoke tests/smoke.cpp)
  target_link_libraries(cityjson_lib_cpp_smoke PRIVATE cityjson_lib_cpp)
  target_compile_definitions(
    cityjson_lib_cpp_smoke
    PRIVATE
      CITYJSON_LIB_FIXTURE_PATH="${CITYJSON_LIB_ROOT_DIR}/tests/data/v2_0/minimal.city.json"
  )
  get_filename_component(CITYJSON_LIB_FFI_CORE_LIB_DIR "${CITYJSON_LIB_FFI_CORE_SHARED_LIB}" DIRECTORY)
  set_target_properties(cityjson_lib_cpp_smoke PROPERTIES BUILD_RPATH "${CITYJSON_LIB_FFI_CORE_LIB_DIR}")
  add_test(NAME cityjson_lib_cpp_smoke COMMAND cityjson_lib_cpp_smoke)
endif()
