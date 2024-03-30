#ifndef PALMDOC_H
#define PALMDOC_H

#include <stddef.h>

/**
 * Compress a null-terminated input string using the PalmDoc compression algorithm.
 *
 * @param input The input string to compress.
 * @param input_len The length of the input string.
 * @param output The output buffer to store the compressed data.
 *
 * @return The length of the compressed output, or 0 on error.
 */
size_t cpalmdoc_compress(const char *input, size_t input_len, char *output);

/**
 * Decompress compressed data using the PalmDoc decompression algorithm.
 *
 * @param input The compressed input data.
 * @param input_len The length of the compressed input data.
 * @param output The output buffer to store the decompressed data.
 *
 * @return The length of the decompressed output, or 0 on error.
 */
size_t cpalmdoc_decompress(const char *input, size_t input_len, char *output);

#endif
