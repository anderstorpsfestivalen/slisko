package null

type Null struct {
}

func (a *Null) Write(pixels []byte) (int, error) {
	return 0, nil
}

func (a *Null) Close() error {
	return nil
}
