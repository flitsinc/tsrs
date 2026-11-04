package main

import (
	"bytes"
	"encoding/json"
	"go/ast"
	"go/parser"
	"go/printer"
	"go/token"
	"os"
	"reflect"
	"strconv"
	"strings"
)

type Field struct {
	Name     string `json:"name"`
	Type     string `json:"type"`
	Embedded bool   `json:"embedded"`
}
type Schema struct {
	Structs map[string][]Field `json:"structs"`
	Aliases map[string]string  `json:"aliases"`
	Methods map[string]string  `json:"methods"`
}

func main() {
	fset := token.NewFileSet()
	file, err := parser.ParseFile(fset, os.Args[len(os.Args)-1], nil, 0)
	if err != nil {
		panic(err)
	}
	exprText := func(expr ast.Expr) string {
		var out bytes.Buffer
		if err := printer.Fprint(&out, fset, expr); err != nil {
			panic(err)
		}
		return out.String()
	}
	result := Schema{Structs: map[string][]Field{}, Aliases: map[string]string{}, Methods: map[string]string{}}
	constants := map[string]string{}
	for _, decl := range file.Decls {
		group, ok := decl.(*ast.GenDecl)
		if !ok {
			continue
		}
		for _, spec := range group.Specs {
			switch spec := spec.(type) {
			case *ast.TypeSpec:
				if object, ok := spec.Type.(*ast.StructType); ok {
					fields := []Field{}
					for _, field := range object.Fields.List {
						fieldType := exprText(field.Type)
						if len(field.Names) == 0 && field.Tag == nil {
							fields = append(fields, Field{Type: fieldType, Embedded: true})
							continue
						}
						if field.Tag == nil {
							continue
						}
						tag, err := strconv.Unquote(field.Tag.Value)
						if err != nil {
							panic(err)
						}
						name := strings.Split(reflect.StructTag(tag).Get("json"), ",")[0]
						if len(field.Names) == 0 && name == "" {
							fields = append(fields, Field{Type: fieldType, Embedded: true})
							continue
						}
						if name == "" || name == "-" {
							continue
						}
						fields = append(fields, Field{Name: name, Type: fieldType})
					}
					result.Structs[spec.Name.Name] = fields
				} else {
					result.Aliases[spec.Name.Name] = exprText(spec.Type)
				}
			case *ast.ValueSpec:
				if group.Tok == token.CONST && len(spec.Names) == 1 && len(spec.Values) == 1 {
					if literal, ok := spec.Values[0].(*ast.BasicLit); ok && literal.Kind == token.STRING {
						text, err := strconv.Unquote(literal.Value)
						if err != nil {
							panic(err)
						}
						constants[spec.Names[0].Name] = text
					}
				}
			}
		}
	}
	ast.Inspect(file, func(node ast.Node) bool {
		spec, ok := node.(*ast.ValueSpec)
		if !ok || len(spec.Names) != 1 || spec.Names[0].Name != "unmarshalers" {
			return true
		}
		table := spec.Values[0].(*ast.CompositeLit)
		for _, item := range table.Elts {
			entry := item.(*ast.KeyValueExpr)
			method := constants[entry.Key.(*ast.Ident).Name]
			if method == "" {
				panic("unknown method constant")
			}
			if generic, ok := entry.Value.(*ast.IndexExpr); ok {
				result.Methods[method] = exprText(generic.Index)
			} else {
				result.Methods[method] = ""
			}
		}
		return false
	})
	if err := json.NewEncoder(os.Stdout).Encode(result); err != nil {
		panic(err)
	}
}
