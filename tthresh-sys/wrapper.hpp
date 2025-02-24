#include <sstream>

#include "tthresh/src/tthresh.hpp"
#include "tthresh/src/compress.hpp"
#include "tthresh/src/decompress.hpp"
#include "tthresh/src/Slice.hpp"

void my_compress(const uint32_t* ds, size_t nd, const char* input, IOType io_type, char** output, size_t* noutput, Target target, double target_value, bool verbose, bool debug) {
    dimensions d;
    for(size_t i = 0; i < nd; ++i) {
        d.s.push_back(ds[i]);
    }
    d.n = d.s.size();
    if (verbose) {
        cout << endl << "/***** Compression: " << to_string(d.n) << "D tensor of size " << d.s[0];
        for (uint8_t i = 1; i < d.n; ++i)
            cout << " x " << d.s[i];
        cout << " *****/" << endl << endl;
    }
    cumulative_products(d.s, d.sprod);

    std::stringstream compressed_stream;

    double *data = compress_stream(d, input, compressed_stream, io_type, target, target_value, verbose, debug);
    delete[] data;

    *noutput = compressed_stream.tellp();

    *output = new char[*noutput];
    compressed_stream.read(*output, *noutput);
}

void dealloc_bytes(char* bytes) {
    delete[] bytes;
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
