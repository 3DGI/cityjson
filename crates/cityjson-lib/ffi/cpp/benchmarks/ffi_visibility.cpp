#include <algorithm>
#include <chrono>
#include <cstdint>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <iterator>
#include <span>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

#include <cityjson_lib/cityjson_lib.hpp>

namespace {

using Clock = std::chrono::steady_clock;
using cityjson_lib::ModelType;
using cityjson_lib::check_status;
using cityjson_lib::to_native;
using cityjson_lib::to_view;
using cityjson_lib::Transform;
using cityjson_lib::WriteOptions;

struct Fixture {
  std::string_view name;
  std::filesystem::path path;
};

struct Mode {
  std::string_view label;
  std::size_t iterations;
  std::size_t repeats;
  std::size_t append_repeats;
};

struct Timing {
  std::uint64_t elapsed_ns;
  double elapsed_per_iteration_ns;
};

struct Result {
  std::string fixture;
  std::string mode;
  std::string operation;
  std::string layer;
  std::size_t iterations;
  std::size_t repeats;
  Timing timing;
};

std::vector<std::uint8_t> read_file_bytes(const std::filesystem::path& path) {
  std::ifstream input(path, std::ios::binary);
  if (!input.good()) {
    throw std::runtime_error("failed to read benchmark fixture: " + path.string());
  }

  return std::vector<std::uint8_t>(
      std::istreambuf_iterator<char>(input), std::istreambuf_iterator<char>());
}

template <typename T>
void do_not_optimize_away(const T& value) {
#if defined(__GNUC__) || defined(__clang__)
  asm volatile("" : : "g"(value) : "memory");
#else
  static_cast<void>(value);
#endif
}

std::uint64_t median(std::vector<std::uint64_t> samples) {
  std::sort(samples.begin(), samples.end());
  if (samples.empty()) {
    return 0U;
  }

  if (samples.size() % 2U == 1U) {
    return samples[samples.size() / 2U];
  }

  const auto upper = samples[samples.size() / 2U];
  const auto lower = samples[(samples.size() / 2U) - 1U];
  return (upper + lower) / 2U;
}

Timing summarize(std::vector<std::uint64_t> samples, std::size_t iterations) {
  const auto elapsed_ns = median(std::move(samples));
  return Timing{
      .elapsed_ns = elapsed_ns,
      .elapsed_per_iteration_ns = static_cast<double>(elapsed_ns) / static_cast<double>(iterations),
  };
}

template <typename Callable>
Timing measure_iterations(std::size_t iterations, std::size_t repeats, Callable&& action) {
  std::vector<std::uint64_t> samples;
  samples.reserve(repeats);
  for (std::size_t repeat = 0U; repeat < repeats; ++repeat) {
    const auto started = Clock::now();
    for (std::size_t iteration = 0U; iteration < iterations; ++iteration) {
      do_not_optimize_away(action());
    }
    samples.push_back(
        static_cast<std::uint64_t>(
            std::chrono::duration_cast<std::chrono::nanoseconds>(Clock::now() - started).count()));
  }
  return summarize(std::move(samples), iterations);
}

template <typename Setup, typename Action, typename Cleanup>
Timing measure_append(std::size_t repeats, Setup&& setup, Action&& action, Cleanup&& cleanup) {
  std::vector<std::uint64_t> samples;
  samples.reserve(repeats);
  for (std::size_t repeat = 0U; repeat < repeats; ++repeat) {
    auto pair = setup();
    try {
      const auto started = Clock::now();
      do_not_optimize_away(action(pair.first, pair.second));
      samples.push_back(
          static_cast<std::uint64_t>(
              std::chrono::duration_cast<std::chrono::nanoseconds>(Clock::now() - started).count()));
    } catch (...) {
      cleanup(pair.first, pair.second);
      throw;
    }
    cleanup(pair.first, pair.second);
  }
  return summarize(std::move(samples), 1U);
}

std::string json_escape(std::string_view value) {
  std::string out;
  out.reserve(value.size() + 8U);
  for (const auto ch : value) {
    switch (ch) {
      case '"':
        out += "\\\"";
        break;
      case '\\':
        out += "\\\\";
        break;
      case '\b':
        out += "\\b";
        break;
      case '\f':
        out += "\\f";
        break;
      case '\n':
        out += "\\n";
        break;
      case '\r':
        out += "\\r";
        break;
      case '\t':
        out += "\\t";
        break;
      default:
        out.push_back(ch);
        break;
    }
  }
  return out;
}

void emit_result(std::ostream& out, const Result& result, bool last) {
  out << "    {\n";
  out << "      \"fixture\": \"" << json_escape(result.fixture) << "\",\n";
  out << "      \"mode\": \"" << json_escape(result.mode) << "\",\n";
  out << "      \"operation\": \"" << json_escape(result.operation) << "\",\n";
  out << "      \"layer\": \"" << json_escape(result.layer) << "\",\n";
  out << "      \"iterations\": " << result.iterations << ",\n";
  out << "      \"repeats\": " << result.repeats << ",\n";
  out << "      \"elapsed_ns\": " << result.timing.elapsed_ns << ",\n";
  out << "      \"elapsed_per_iteration_ns\": " << result.timing.elapsed_per_iteration_ns << "\n";
  out << "    }" << (last ? "" : ",") << "\n";
}

Result make_result(
    std::string_view fixture,
    std::string_view mode,
    std::string_view operation,
    std::string_view layer,
    std::size_t iterations,
    std::size_t repeats,
    Timing timing) {
  return Result{
      .fixture = std::string(fixture),
      .mode = std::string(mode),
      .operation = std::string(operation),
      .layer = std::string(layer),
      .iterations = iterations,
      .repeats = repeats,
      .timing = std::move(timing),
  };
}

std::vector<Result> benchmark_fixture(std::string_view fixture_name, std::span<const std::uint8_t> bytes, const Mode& mode) {
  std::vector<Result> results;
  results.reserve(10U);

  const auto wrapper_parse = measure_iterations(mode.iterations, mode.repeats, [&] {
    const auto model = cityjson_lib::Model::parse_document(bytes);
    const auto summary = model.summary();
    do_not_optimize_away(summary.cityobject_count);
    return summary.cityobject_count;
  });
  const auto abi_parse = measure_iterations(mode.iterations, mode.repeats, [&] {
    cj_model_t* handle = nullptr;
    check_status(cj_model_parse_document_bytes(bytes.data(), bytes.size(), &handle));
    cj_model_summary_t summary{};
    check_status(cj_model_get_summary(handle, &summary));
    do_not_optimize_away(summary.cityobject_count);
    check_status(cj_model_free(handle));
    return summary.cityobject_count;
  });

  const auto wrapper_model = cityjson_lib::Model::parse_document(bytes);
  cj_model_t* abi_handle = nullptr;
  check_status(cj_model_parse_document_bytes(bytes.data(), bytes.size(), &abi_handle));
  const WriteOptions serialize_options{false, false};

  const auto wrapper_serialize = measure_iterations(mode.iterations, mode.repeats, [&] {
    const auto output = wrapper_model.serialize_document_bytes(serialize_options);
    do_not_optimize_away(output.size());
    return output.size();
  });
  const auto abi_serialize = measure_iterations(mode.iterations, mode.repeats, [&] {
    cj_bytes_t bytes_out{};
    check_status(cj_model_serialize_document_with_options(
        abi_handle, to_native(serialize_options), &bytes_out));
    const auto output = cityjson_lib::take_bytes(bytes_out);
    do_not_optimize_away(output.size());
    return output.size();
  });

  const auto wrapper_cityobject_ids = measure_iterations(mode.iterations, mode.repeats, [&] {
    const auto ids = wrapper_model.cityobject_ids();
    do_not_optimize_away(ids.size());
    return ids.size();
  });
  const auto abi_cityobject_ids = measure_iterations(mode.iterations, mode.repeats, [&] {
    cj_bytes_list_t ids{};
    check_status(cj_model_copy_cityobject_ids(abi_handle, &ids));
    const auto output = cityjson_lib::take_string_list(ids);
    do_not_optimize_away(output.size());
    return output.size();
  });

  const auto wrapper_geometry_types = measure_iterations(mode.iterations, mode.repeats, [&] {
    const auto types = wrapper_model.geometry_types();
    do_not_optimize_away(types.size());
    return types.size();
  });
  const auto abi_geometry_types = measure_iterations(mode.iterations, mode.repeats, [&] {
    cj_geometry_types_t types{};
    check_status(cj_model_copy_geometry_types(abi_handle, &types));
    const auto output = cityjson_lib::take_geometry_types(types);
    do_not_optimize_away(output.size());
    return output.size();
  });

  check_status(cj_model_free(abi_handle));

  results.push_back(make_result(fixture_name, mode.label, "parse", "wrapper", mode.iterations, mode.repeats, wrapper_parse));
  results.push_back(make_result(fixture_name, mode.label, "parse", "abi", mode.iterations, mode.repeats, abi_parse));
  results.push_back(make_result(fixture_name, mode.label, "serialize", "wrapper", mode.iterations, mode.repeats, wrapper_serialize));
  results.push_back(make_result(fixture_name, mode.label, "serialize", "abi", mode.iterations, mode.repeats, abi_serialize));
  results.push_back(make_result(fixture_name, mode.label, "cityobject_ids", "wrapper", mode.iterations, mode.repeats, wrapper_cityobject_ids));
  results.push_back(make_result(fixture_name, mode.label, "cityobject_ids", "abi", mode.iterations, mode.repeats, abi_cityobject_ids));
  results.push_back(make_result(fixture_name, mode.label, "geometry_types", "wrapper", mode.iterations, mode.repeats, wrapper_geometry_types));
  results.push_back(make_result(fixture_name, mode.label, "geometry_types", "abi", mode.iterations, mode.repeats, abi_geometry_types));

  return results;
}

void clear_cityobject_geometries(cityjson_lib::Model& model);
void clear_cityobject_geometries(cj_model_t* model);

std::vector<Result> benchmark_append(const Mode& mode) {
  const auto source_bytes = read_file_bytes(CITYJSON_LIB_SMALL_FIXTURE_PATH);

  const auto wrapper_append = measure_append(mode.append_repeats, [&] {
    auto target = cityjson_lib::Model::create(ModelType::CJ_MODEL_TYPE_CITY_JSON);
    target.set_transform(Transform{});
    auto source = cityjson_lib::Model::parse_document(source_bytes);
    source.set_transform(Transform{});
    clear_cityobject_geometries(source);
    return std::pair{std::move(target), std::move(source)};
  }, [&](cityjson_lib::Model& target, cityjson_lib::Model& source) {
    target.append_model(source);
    const auto summary = target.summary();
    do_not_optimize_away(summary.cityobject_count);
    return summary.cityobject_count;
  }, [](auto&, auto&) {});
  const auto abi_append = measure_append(mode.append_repeats, [&] {
    cj_model_t* target = nullptr;
    check_status(cj_model_create(CJ_MODEL_TYPE_CITY_JSON, &target));
    check_status(cj_model_set_transform(target, to_native(Transform{})));
    cj_model_t* source = nullptr;
    check_status(cj_model_parse_document_bytes(source_bytes.data(), source_bytes.size(), &source));
    check_status(cj_model_set_transform(source, to_native(Transform{})));
    clear_cityobject_geometries(source);
    return std::pair{target, source};
  }, [&](cj_model_t*& target, cj_model_t*& source) {
    check_status(cj_model_append_model(target, source));
    cj_model_summary_t summary{};
    check_status(cj_model_get_summary(target, &summary));
    do_not_optimize_away(summary.cityobject_count);
    return summary.cityobject_count;
  }, [&](cj_model_t*& target, cj_model_t*& source) {
    check_status(cj_model_free(target));
    check_status(cj_model_free(source));
  });

  std::vector<Result> results;
  results.push_back(make_result(
      "empty<-small",
      mode.label,
      "append",
      "wrapper",
      1U,
      mode.append_repeats,
      wrapper_append));
  results.push_back(make_result(
      "empty<-small",
      mode.label,
      "append",
      "abi",
      1U,
      mode.append_repeats,
      abi_append));
  return results;
}

void clear_cityobject_geometries(cityjson_lib::Model& model) {
  for (const auto& cityobject_id : model.cityobject_ids()) {
    check_status(cj_model_clear_cityobject_geometry(model.raw_handle(), to_view(cityobject_id)));
  }
}

void clear_cityobject_geometries(cj_model_t* model) {
  cj_bytes_list_t ids{};
  check_status(cj_model_copy_cityobject_ids(model, &ids));
  const auto values = cityjson_lib::take_string_list(ids);
  for (const auto& cityobject_id : values) {
    check_status(cj_model_clear_cityobject_geometry(model, to_view(cityobject_id)));
  }
}

std::vector<Result> benchmark(const Mode& mode) {
  const Fixture fixtures[] = {
      {
          .name = "small",
          .path = CITYJSON_LIB_SMALL_FIXTURE_PATH,
      },
      {
          .name = "medium",
          .path = CITYJSON_LIB_LARGE_FIXTURE_PATH,
      },
  };

  std::vector<Result> results;
  for (const auto& fixture : fixtures) {
    const auto bytes = read_file_bytes(fixture.path);
    const auto fixture_results = benchmark_fixture(fixture.name, bytes, mode);
    results.insert(results.end(), fixture_results.begin(), fixture_results.end());
  }
  const auto append_results = benchmark_append(mode);
  results.insert(results.end(), append_results.begin(), append_results.end());
  return results;
}

void print_report(const Mode& mode, const std::vector<Result>& results) {
  std::cout << "{\n";
  std::cout << "  \"language\": \"cpp\",\n";
  std::cout << "  \"mode\": \"" << json_escape(mode.label) << "\",\n";
  std::cout << "  \"results\": [\n";
  for (std::size_t index = 0U; index < results.size(); ++index) {
    emit_result(std::cout, results[index], index + 1U == results.size());
  }
  std::cout << "  ]\n";
  std::cout << "}\n";
}

}  // namespace

int main(int argc, char** argv) {
  bool quick = false;
  for (int index = 1; index < argc; ++index) {
    if (std::string_view(argv[index]) == "--quick") {
      quick = true;
      break;
    }
  }

  const Mode mode = quick
      ? Mode{.label = "quick", .iterations = 250U, .repeats = 7U, .append_repeats = 11U}
      : Mode{.label = "full", .iterations = 1000U, .repeats = 9U, .append_repeats = 15U};

  try {
    const auto results = benchmark(mode);
    print_report(mode, results);
  } catch (const std::exception& error) {
    std::cerr << error.what() << '\n';
    return 1;
  }

  return 0;
}
