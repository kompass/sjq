# sjq

*sjq* is a lightweight and fast processor for huge files or streams. Performance and limited memory use are a priority. It is inspired by jq, but with better streaming.

The idea is simple : parse only what we need, stream data, chain processes, output as soon as possible.

## Usage

```
sjq [FLAGS] [OPTIONS] <query>

FLAGS:
    -h, --help       Prints help information
    -p, --pretty     
    -V, --version    Prints version information

OPTIONS:
    -o, --output <output>
```