package callcheck

import (
	"reflect"

	"github.com/pkg/errors"
)

// Predicate calls a method `methodName` on the Reciever `recv`.
func Predicate(recv interface{}, methodName string) (bool, error) {
	val := reflect.ValueOf(recv)
	typ := reflect.TypeOf(recv)
	method, ok := typ.MethodByName(methodName)
	if !ok {
		return false, errors.Errorf("no predicate method named %q", methodName)
	}
	res := method.Func.Call([]reflect.Value{val})
	if len(res) != 1 {
		return false, errors.Errorf("expected single return value from predicate method")
	}
	if res[0].Type().Name() != "bool" {
		return false, errors.Errorf("return value from predicate was not a bool")
	}
	return res[0].Bool(), nil
}
