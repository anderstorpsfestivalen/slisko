package main

import (
	"fmt"

	"github.com/anderstorpsfestivalen/slisko/pkg/chassi"
)

func main() {
	d := chassi.Gen6704()

	d.Link[1].G = 1.0
	fmt.Println(d)
}
