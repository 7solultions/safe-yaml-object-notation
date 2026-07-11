package syon

import (
	"errors"
	"fmt"
	"reflect"
	"strconv"
	"strings"
)

// Unmarshal parses SYON and stores the result in the value pointed to by v.
//
// Because SYON scalars are strings, Unmarshal coerces them to the target field's
// type: "true"/"false" → bool, digits → int/uint/float, otherwise string. Struct
// fields are matched by a `syon:"name"` tag, then a `yaml:"name"` tag (to ease
// migration), then the lower-cased field name. Unknown keys are ignored.
func Unmarshal(data []byte, v any) error {
	node, err := Parse(data)
	if err != nil {
		return err
	}
	rv := reflect.ValueOf(v)
	if rv.Kind() != reflect.Pointer || rv.IsNil() {
		return errors.New("syon: Unmarshal requires a non-nil pointer")
	}
	return decode(node, rv.Elem())
}

func decode(n *Node, rv reflect.Value) error {
	for rv.Kind() == reflect.Pointer {
		if rv.IsNil() {
			rv.Set(reflect.New(rv.Type().Elem()))
		}
		rv = rv.Elem()
	}

	switch rv.Kind() {
	case reflect.Interface:
		rv.Set(reflect.ValueOf(generic(n)))
		return nil

	case reflect.String:
		rv.SetString(scalarStr(n))
		return nil

	case reflect.Bool:
		s := scalarStr(n)
		switch s {
		case "true":
			rv.SetBool(true)
		case "false":
			rv.SetBool(false)
		default:
			return &Error{n.Line, 1, "syntax", fmt.Sprintf("cannot use %q as bool (want true/false)", s)}
		}
		return nil

	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		i, err := strconv.ParseInt(scalarStr(n), 10, 64)
		if err != nil {
			return &Error{n.Line, 1, "syntax", fmt.Sprintf("cannot use %q as integer", scalarStr(n))}
		}
		rv.SetInt(i)
		return nil

	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		u, err := strconv.ParseUint(scalarStr(n), 10, 64)
		if err != nil {
			return &Error{n.Line, 1, "syntax", fmt.Sprintf("cannot use %q as unsigned integer", scalarStr(n))}
		}
		rv.SetUint(u)
		return nil

	case reflect.Float32, reflect.Float64:
		f, err := strconv.ParseFloat(scalarStr(n), 64)
		if err != nil {
			return &Error{n.Line, 1, "syntax", fmt.Sprintf("cannot use %q as float", scalarStr(n))}
		}
		rv.SetFloat(f)
		return nil

	case reflect.Slice:
		if isEmptyScalar(n) { // SYON has no `[]`; an empty value is an empty list
			rv.Set(reflect.MakeSlice(rv.Type(), 0, 0))
			return nil
		}
		if n.Kind != SequenceNode {
			return &Error{n.Line, 1, "syntax", "expected a sequence"}
		}
		s := reflect.MakeSlice(rv.Type(), len(n.Seq), len(n.Seq))
		for i, item := range n.Seq {
			if err := decode(item, s.Index(i)); err != nil {
				return err
			}
		}
		rv.Set(s)
		return nil

	case reflect.Map:
		if isEmptyScalar(n) {
			rv.Set(reflect.MakeMap(rv.Type()))
			return nil
		}
		if n.Kind != MappingNode {
			return &Error{n.Line, 1, "syntax", "expected a mapping"}
		}
		m := reflect.MakeMapWithSize(rv.Type(), len(n.Keys))
		for _, k := range n.Keys {
			elem := reflect.New(rv.Type().Elem()).Elem()
			if err := decode(n.Map[k], elem); err != nil {
				return err
			}
			m.SetMapIndex(reflect.ValueOf(k).Convert(rv.Type().Key()), elem)
		}
		rv.Set(m)
		return nil

	case reflect.Struct:
		if n.Kind != MappingNode {
			return &Error{n.Line, 1, "syntax", "expected a mapping"}
		}
		return decodeStruct(n, rv)
	}
	return fmt.Errorf("syon: cannot decode into %s", rv.Kind())
}

func decodeStruct(n *Node, rv reflect.Value) error {
	t := rv.Type()
	fields := map[string]int{}
	for i := 0; i < t.NumField(); i++ {
		f := t.Field(i)
		if f.PkgPath != "" { // unexported
			continue
		}
		name, ok := fieldKey(f)
		if ok {
			fields[name] = i
		}
	}
	for _, key := range n.Keys {
		idx, ok := fields[key]
		if !ok {
			continue // unknown key — ignore
		}
		if err := decode(n.Map[key], rv.Field(idx)); err != nil {
			return err
		}
	}
	return nil
}

func fieldKey(f reflect.StructField) (string, bool) {
	for _, tag := range []string{"syon", "yaml"} {
		if v := f.Tag.Get(tag); v != "" {
			name := strings.Split(v, ",")[0]
			if name == "-" {
				return "", false
			}
			if name != "" {
				return name, true
			}
		}
	}
	return strings.ToLower(f.Name), true
}

func isEmptyScalar(n *Node) bool {
	return n.Kind == ScalarNode && n.Str == ""
}

func scalarStr(n *Node) string {
	switch n.Kind {
	case ScalarNode, LiteralNode, FenceNode:
		return n.Str
	default:
		return ""
	}
}

// generic converts a Node into plain Go values (for interface{} targets).
func generic(n *Node) any {
	switch n.Kind {
	case MappingNode:
		m := make(map[string]any, len(n.Keys))
		for _, k := range n.Keys {
			m[k] = generic(n.Map[k])
		}
		return m
	case SequenceNode:
		s := make([]any, len(n.Seq))
		for i, item := range n.Seq {
			s[i] = generic(item)
		}
		return s
	default:
		return n.Str
	}
}
