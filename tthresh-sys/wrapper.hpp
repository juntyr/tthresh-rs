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

void my_decompress(const uint32_t* ds, size_t nd, const uint8_t* input, size_t ninput, char** output, size_t* noutput, IOType* io_type, bool verbose, bool debug) {
    dimensions d;
    for(size_t i = 0; i < nd; ++i) {
        d.s.push_back(ds[i]);
    }

    vector<Slice> cutout;

    std::stringstream compressed_stream(std::string(reinterpret_cast<const char*>(input), ninput));
    std::stringstream decompressed_stream;

    *io_type = decompress_stream(d, compressed_stream, decompressed_stream, nullptr, cutout, false, verbose, debug);

    *noutput = decompressed_stream.tellp();

    *output = new char[*noutput];
    decompressed_stream.read(*output, *noutput);
}
