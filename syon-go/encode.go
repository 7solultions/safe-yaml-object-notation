package syon

import (
	"fmt"
	"reflect"
	"sort"
	"strconv"
	"strings"
)

// Marshal serializes a Go value into SYON text.
//
// Structs and maps become mappings (struct field order; map keys sorted);
// slices become sequences; scalars become string values, quoted only when
// needed. Multi-line strings are written as `[[[ … ]]]` literal blocks. The
// output is valid SYON that Unmarshal round-trips.
func Marshal(v any) ([]byte, error) {
	var b strings.Builder
	rv := deref(reflect.ValueOf(v))
	var err error
	switch rv.Kind() {
	case reflect.Struct, reflect.Map:
		err = encodeMapping(&b, mappingEntries(rv), 0)
	case reflect.Slice, reflect.Array:
		err = encodeSequence(&b, rv, 0)
	default:
		b.WriteString(quoteScalar(scalarText(rv)) + "\n")
	}
	if err != nil {
		return nil, err
	}
	return []byte(b.String()), nil
}

type entry struct {
	key string
	val reflect.Value
}

func mappingEntries(rv reflect.Value) []entry {
	var out []entry
	switch rv.Kind() {
	case reflect.Struct:
		t := rv.Type()
		for i := 0; i < t.NumField(); i++ {
			f := t.Field(i)
			if f.PkgPath != "" {
				continue
			}
			name, ok := fieldKey(f)
			if !ok {
				continue
			}
			out = append(out, entry{name, rv.Field(i)})
		}
	case reflect.Map:
		keys := rv.MapKeys()
		sort.Slice(keys, func(i, j int) bool {
			return fmt.Sprint(keys[i].Interface()) < fmt.Sprint(keys[j].Interface())
		})
		for _, k := range keys {
			out = append(out, entry{fmt.Sprint(k.Interface()), rv.MapIndex(k)})
		}
	}
	return out
}

func encodeMapping(b *strings.Builder, entries []entry, depth int) error {
	ind := strings.Repeat("  ", depth)
	for _, e := range entries {
		v := deref(e.val)
		switch v.Kind() {
		case reflect.Struct, reflect.Map:
			sub := mappingEntries(v)
			if len(sub) == 0 {
				b.WriteString(ind + e.key + ":\n")
				continue
			}
			b.WriteString(ind + e.key + ":\n")
			if err := encodeMapping(b, sub, depth+1); err != nil {
				return err
			}
		case reflect.Slice, reflect.Array:
			if v.Len() == 0 {
				b.WriteString(ind + e.key + ":\n")
				continue
			}
			b.WriteString(ind + e.key + ":\n")
			if err := encodeSequence(b, v, depth+1); err != nil {
				return err
			}
		default:
			s := scalarText(v)
			if strings.Contains(s, "\n") {
				writeLiteral(b, ind, e.key+":", s)
			} else {
				b.WriteString(ind + e.key + ": " + quoteScalar(s) + "\n")
			}
		}
	}
	return nil
}

func encodeSequence(b *strings.Builder, rv reflect.Value, depth int) error {
	ind := strings.Repeat("  ", depth)
	for i := 0; i < rv.Len(); i++ {
		item := deref(rv.Index(i))
		switch item.Kind() {
		case reflect.Struct, reflect.Map:
			b.WriteString(ind + "-\n")
			if err := encodeMapping(b, mappingEntries(item), depth+1); err != nil {
				return err
			}
		case reflect.Slice, reflect.Array:
			b.WriteString(ind + "-\n")
			if err := encodeSequence(b, item, depth+1); err != nil {
				return err
			}
		default:
			s := scalarText(item)
			if strings.Contains(s, "\n") {
				writeLiteral(b, ind, "-", s)
			} else {
				b.WriteString(ind + "- " + quoteScalar(s) + "\n")
			}
		}
	}
	return nil
}

// writeLiteral emits `prefix [[[` then the dedented content re-indented under
// the block, then `]]]`.
func writeLiteral(b *strings.Builder, ind, prefix, s string) {
	b.WriteString(ind + prefix + " [[[\n")
	for _, line := range strings.Split(s, "\n") {
		if line == "" {
			b.WriteString("\n")
		} else {
			b.WriteString(ind + "  " + line + "\n")
		}
	}
	b.WriteString(ind + "]]]\n")
}

func scalarText(rv reflect.Value) string {
	switch rv.Kind() {
	case reflect.Bool:
		if rv.Bool() {
			return "true"
		}
		return "false"
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		return strconv.FormatInt(rv.Int(), 10)
	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		return strconv.FormatUint(rv.Uint(), 10)
	case reflect.Float32, reflect.Float64:
		return strconv.FormatFloat(rv.Float(), 'g', -1, 64)
	case reflect.String:
		return rv.String()
	default:
		return fmt.Sprint(rv.Interface())
	}
}

// quoteScalar double-quotes a value only when a bare scalar would be misparsed.
func quoteScalar(s string) string {
	if s == "" {
		return `""`
	}
	needs := s != strings.TrimSpace(s) // leading/trailing whitespace
	switch s[0] {
	case '"', '#', '{', '[', '!', '&', '*', '?':
		needs = true
	}
	if strings.HasPrefix(s, "# ") || strings.Contains(s, " #") {
		needs = true
	}
	if !needs {
		return s
	}
	esc := strings.NewReplacer(`\`, `\\`, `"`, `\"`, "\n", `\n`, "\t", `\t`).Replace(s)
	return `"` + esc + `"`
}

func deref(rv reflect.Value) reflect.Value {
	for rv.Kind() == reflect.Pointer || rv.Kind() == reflect.Interface {
		if rv.IsNil() {
			return rv
		}
		rv = rv.Elem()
	}
	return rv
}
