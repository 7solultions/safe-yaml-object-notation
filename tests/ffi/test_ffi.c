#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Symbols exported by libsyon_parser */
extern char *syon_parse_json(const char *input);
extern void  syon_free(char *ptr);

static int check(const char *label, const char *json, const char *expected_key) {
    if (strstr(json, expected_key) == NULL) {
        fprintf(stderr, "FAIL [%s]: expected substring %s in: %s\n",
                label, expected_key, json);
        return 1;
    }
    printf("PASS [%s]: %s\n", label, json);
    return 0;
}

int main(void) {
    int failures = 0;
    char *json;

    /* basic key-value parse */
    json = syon_parse_json("name: beriah\nversion: 1.0\n");
    failures += check("basic", json, "beriah");
    syon_free(json);

    /* url with colon-no-space stays literal */
    json = syon_parse_json("url: http://example.com\n");
    failures += check("url-colon", json, "http://example.com");
    syon_free(json);

    /* forbidden YAML tag returns error object */
    json = syon_parse_json("value: !!str hello\n");
    failures += check("forbidden-tag-error", json, "error");
    syon_free(json);

    /* null pointer input returns error object */
    json = syon_parse_json(NULL);
    failures += check("null-input-error", json, "error");
    syon_free(json);

    if (failures == 0) {
        printf("All FFI tests passed.\n");
    }
    return failures;
}
