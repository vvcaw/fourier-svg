# fourier-svg
Illustrating how to approximate svg files using the discrete fourier transform.
![image](https://user-images.githubusercontent.com/57096338/177039807-abe37d9e-012f-407a-911f-d1200cde5a80.png)

## How to run
Make sure **Rust** and **Cargo** are installed on your system.

```
git clone https://github.com/vvcaw/fourier-svg.git --recursive
cd fourier-svg
```

**Run** the project
```
cargo run --release -- -f fourier.svg
```

or **build** the executable.
```
cargo build --release
./target/release/fourier-svg -f fourier.svg
```

### Options
Use the `-h` flag to get help concerning all options.
```
fourier-svg 0.1.0

USAGE:
    fourier-svg [OPTIONS] --file <file>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --distance <distance>    Distance between points sampled from svg [default: 5.0]
    -f, --file <file>            Input svg file
```

## Contributing
In case you find bugs or want to improve the codebase, `pull requests` are welcome.
