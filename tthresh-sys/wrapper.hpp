#include "tthresh/src/tthresh.hpp"
#include "tthresh/src/compress.hpp"
#include "tthresh/src/decompress.hpp"
#include "tthresh/src/Slice.hpp"

void my_compress(const uint32_t* ds, size_t nd, const char* input_file, const char* compressed_file, const char* io_type, Target target, double target_value, bool verbose=false, bool debug=false) {
    dimensions d;
    for(size_t i = 0; i < nd; ++i) {
        d.s.push_back(ds[i]);
    }
    if (d.s.size() < 3) {
        display_error("Specify 3 or more integer sizes after -s (C memory order)");
    }

    double *data = compress(d, std::string(input_file), std::string(compressed_file), std::string(io_type), target, target_value, 0, verbose, debug);
    delete[] data;
}

void my_decompress(const uint32_t* ds, size_t nd, const char* compressed_file, const char* output_file, bool verbose, bool debug) {
    dimensions d;
    for(size_t i = 0; i < nd; ++i) {
        d.s.push_back(ds[i]);
    }
    if (d.s.size() < 3) {
        display_error("Specify 3 or more integer sizes after -s (C memory order)");
    }

    vector<Slice> cutout;

    decompress(d, std::string(compressed_file), std::string(output_file), nullptr, cutout, false, verbose, debug);
}
