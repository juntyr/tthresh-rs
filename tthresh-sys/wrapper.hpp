#include <sstream>

#include "tthresh/src/tthresh.hpp"
#include "tthresh/src/compress.hpp"
#include "tthresh/src/decompress.hpp"
#include "tthresh/src/Slice.hpp"

typedef void* (*alloc)(size_t /*size*/, size_t /*align*/);

void compress_buffer(
    // input buffer and type
    const char* input, IOType input_type,
    // input shape
    const uint32_t* shape, size_t shape_size,
    // output byte buffer
    uint8_t** output, size_t* output_size,
    // compression error target
    Target target, double target_value,
    // allocator for the output
    alloc alloc,
    // debugging
    bool verbose, bool debug
) {
    // create the dimensions from the shape
    dimensions dims;
    for(size_t i = 0; i < shape_size; ++i) {
        dims.s.push_back(shape[i]);
    }
    dims.n = dims.s.size();
    cumulative_products(dims.s, dims.sprod);

    if (verbose) {
        std::cout << endl << "/***** Compression: " << to_string(dims.n) << "D tensor of size " << dims.s[0];
        for (uint8_t i = 1; i < dims.n; ++i)
            std::cout << " x " << dims.s[i];
        std::cout << " *****/" << std::endl << std::endl;
    }

    std::stringstream compressed_stream(std::ios::in | std::ios::out | std::ios::binary);

    // compress the input data into the compressed output stream
    double *data = compress_stream(
        dims, input, compressed_stream, input_type, target, target_value, verbose, debug
    );

    // delete the data if it is a copy (and doesn't point to input)
    if (reinterpret_cast<char*>(data) != input) {
        delete[] data;
    }

    // extract the size of the compressed data
    compressed_stream.seekp(0, std::ios::end);
    *output_size = compressed_stream.tellp();

    // copy the compressed data into a fresh output byte buffer
    *output = reinterpret_cast<uint8_t*>(
        alloc(sizeof(uint8_t) * (*output_size), alignof(uint8_t))
    );
    compressed_stream.seekg(0, std::ios::beg);
    compressed_stream.read(reinterpret_cast<char*>(*output), *output_size);
}

void decompress_buffer(
    // input byte buffer
    const uint8_t* input, size_t input_size,
    // output buffer, type, and length (in units of the type)
    char** output, IOType* output_type, size_t* output_length,
    // output shape
    uint32_t** shape, size_t* shape_size,
    // allocator for the output
    alloc alloc,
    // debugging
    bool verbose, bool debug
) {
    dimensions dims;
    vector<Slice> cutout;

    std::istringstream compressed_stream(
        std::string(reinterpret_cast<const char*>(input), input_size),
        std::ios::in | std::ios::binary
    );
    std::stringstream decompressed_stream(std::ios::in | std::ios::out | std::ios::binary);

    // decompress the compressed input stream into the decompressed output stream
    *output_type = decompress_stream(
        dims, compressed_stream, decompressed_stream, nullptr, cutout, false, verbose, debug
    );

    // copy the output shape into a fresh shape buffer
    *shape_size = dims.n;
    *shape = reinterpret_cast<uint32_t*>(
        alloc(sizeof(uint32_t) * (*shape_size), alignof(uint32_t))
    );
    for(size_t i = 0; i < *shape_size; ++i) {
        (*shape)[i] = dims.s[i];
    }

    // extract the size of the output type
    size_t output_type_size, output_type_align;
    switch (*output_type) {
        case IOType::uchar_: {
            output_type_size = sizeof(unsigned char);
            output_type_align = alignof(unsigned char);
            break;
        }
        case IOType::ushort_: {
            output_type_size = sizeof(unsigned short);
            output_type_align = alignof(unsigned short);
            break;
        }
        case IOType::int_: {
            output_type_size = sizeof(int);
            output_type_align = alignof(int);
            break;
        }
        case IOType::float_: {
            output_type_size = sizeof(float);
            output_type_align = alignof(float);
            break;
        }
        case IOType::double_: {
            output_type_size = sizeof(double);
            output_type_align = alignof(double);
            break;
        }
    }

    *output_length = dims.sprod[dims.n];

    // copy the decompressed output into a fresh buffer
    *output = reinterpret_cast<char*>(
        alloc(output_type_size * (*output_length), output_type_align)
    );
    decompressed_stream.seekg(0, std::ios::beg);
    decompressed_stream.read(*output, *output_length * output_type_size);
}
