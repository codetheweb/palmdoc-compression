// This was taken from Calibre's source at commit e85af24.
// I slightly modified it by removing the Python bindings.
#include <stdio.h>
#include <stdlib.h>

#define BUFFER 6000

#define MIN(x, y) ( ((x) < (y)) ? (x) : (y) )
#define MAX(x, y) ( ((x) > (y)) ? (x) : (y) )

typedef unsigned short int Byte;
typedef struct {
    Byte *data;
    size_t len;
} buffer;

#ifdef bool
#undef bool
#endif
#define bool int

#ifdef false
#undef false
#endif
#define false 0

#ifdef true
#undef true
#endif
#define true 1

#define CHAR(x) (( (x) > 127 ) ? (x)-256 : (x))

static bool cpalmdoc_memcmp(Byte *a, Byte *b, size_t len) {
    size_t i;
    for (i = 0; i < len; i++) if (a[i] != b[i]) return false;
    return true;
}

static size_t cpalmdoc_rfind(Byte *data, size_t pos, size_t chunk_length) {
    size_t i;
    for (i = pos - chunk_length; i > 0; i--) {
        if (cpalmdoc_memcmp(data+i, data+pos, chunk_length)) return i;
    }
    return pos;
}

static size_t cpalmdoc_do_compress(buffer *b, char *output) {
    size_t i = 0, j, chunk_len, dist;
    unsigned int compound;
    Byte c, n;
    bool found;
    char *head;
    buffer temp;
    head = output;
    temp.data = (Byte *)malloc(sizeof(Byte)*8); temp.len = 0;
    if (temp.data == NULL) return 0;
    while (i < b->len) {
        c = b->data[i];
        //do repeats
        if ( i > 10 && (b->len - i) > 10) {
            found = false;
            for (chunk_len = 10; chunk_len > 2; chunk_len--) {
                j = cpalmdoc_rfind(b->data, i, chunk_len);
                dist = i - j;
                if (j < i && dist <= 2047) {
                    found = true;
                    compound = (unsigned int)((dist << 3) + chunk_len-3);
                    *(output++) = CHAR(0x80 + (compound >> 8 ));
                    *(output++) = CHAR(compound & 0xFF);
                    i += chunk_len;
                    break;
                }
            }
            if (found) continue;
        }

        //write single character
        i++;
        if (c == 32 && i < b->len) {
            n = b->data[i];
            if ( n >= 0x40 && n <= 0x7F) {
                *(output++) = CHAR(n^0x80); i++; continue;
            }
        }
        if (c == 0 || (c > 8 && c < 0x80))
            *(output++) = CHAR(c);
        else { // Write binary data
            j = i;
            temp.data[0] = c; temp.len = 1;
            while (j < b->len && temp.len < 8) {
                c = b->data[j];
                if (c == 0 || (c > 8 && c < 0x80)) break;
                temp.data[temp.len++] = c; j++;
            }
            i += temp.len - 1;
            *(output++) = (char)temp.len;
            for (j=0; j < temp.len; j++) *(output++) = (char)temp.data[j];
        }
    }
    free(temp.data);
    return output - head;
}

size_t cpalmdoc_compress(char *input, size_t input_len, char *output) {
    size_t j = 0;
    buffer b;
    b.data = (Byte *)malloc(sizeof(Byte)*input_len);
    if (b.data == NULL) return 0;
    // Map chars to bytes
    for (j = 0; j < input_len; j++)
        b.data[j] = (input[j] < 0) ? input[j]+256 : input[j];
    b.len = input_len;
    j = cpalmdoc_do_compress(&b, output);
    free(b.data);
    return j;
}

size_t cpalmdoc_decompress(char *input, size_t input_len, char *output) {
    Byte *in_bytes; Byte c;
    size_t i = 0, j = 0, o = 0, di = 0, n = 0;
    in_bytes = (Byte *)malloc(sizeof(Byte)*input_len);
    if (in_bytes == NULL) return 0;
    // Map chars to bytes
    for (j = 0; j < input_len; j++) {
        in_bytes[j] = (input[j] < 0) ? input[j]+256 : input[j];
    }

    while (i < input_len) {
        c = in_bytes[i++];
        if (c >= 1 && c <= 8)  { // copy 'c' bytes
            while (c-- && i < input_len) { output[o++] = in_bytes[i++]; }
        } else if (c <= 0x7F)  { // 0, 09-7F = self
            output[o++] = (char)c;
        } else if (c >= 0xC0) { // space + ASCII char
            output[o++] = ' '; output[o++] = c ^ 0x80;
        } else if (i < input_len) { // 80-BF repeat sequences
            c = (c << 8) + in_bytes[i++];
            di = (c & 0x3FFF) >> 3;
            if (di <= o) {
                for ( n = (c & 7) + 3; n--; ++o ) {
                    output[o] = output[o-di];
                }
            }
        }
    }
    free(in_bytes);
    output[o] = '\0';
    return o;
}
