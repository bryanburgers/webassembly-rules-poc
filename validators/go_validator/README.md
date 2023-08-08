Build:

```
docker run --rm -v $(pwd):/home/tinygo/go/go_validator -w /home/tinygo/go/go_validator tinygo/tinygo:0.28.1 tinygo build -o go.wasm -target=wasm
```
