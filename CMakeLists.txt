cmake_minimum_required(VERSION 4.0)

find_package(cmake-bare REQUIRED PATHS node_modules/cmake-bare)
find_package(cmake-cargo REQUIRED PATHS node_modules/cmake-cargo)
find_package(cmake-fetch REQUIRED PATHS node_modules/cmake-fetch)
find_package(cmake-napi REQUIRED PATHS node_modules/cmake-napi)

project(bare_bundle_lower C)

fetch_package("github:holepunchto/libnapish#8cbfe78")

add_crate()

add_bare_module(bare_bundle_lower)

target_sources(
  ${bare_bundle_lower}
  PRIVATE
    binding.c
)

target_link_libraries(
  ${bare_bundle_lower}
  PRIVATE
    bare_bundle_lower
)

add_napi_module(bare_bundle_lower_napi)

target_sources(
  ${bare_bundle_lower_napi}
  PRIVATE
    binding.c
)

target_link_libraries(
  ${bare_bundle_lower_napi}
  PRIVATE
    bare_bundle_lower
    napish::static
)
