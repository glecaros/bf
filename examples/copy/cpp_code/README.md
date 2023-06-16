## Example CPP project

Building your `.so` file
```bash
c++ -fPIC -c shared.cpp -o build/shared.o
c++ -shared  -Wl,-soname,libshared.so -o build/libshared.so build/shared.o
```

Building main using your `.so` file

```bash
c++ -o main  build/main.o -L. -lshared
```
