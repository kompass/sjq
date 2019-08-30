# sjq

*Warning :* This tool is still on alpha, nothing works correctly yet !

*sjq* is a lightweight and fast processor for huge files or streams. Performance and limited memory use are a priority. It is inspired by jq, but with better streaming.

The idea is simple : parse only what we need, stream data, chain processes, output as soon as possible.

## Usage

```
USAGE:
    sjq [FLAGS] [OPTIONS] <query>

FLAGS:
    -a, --append       If output filename specified, appends instead of overwriting previous content
    -f, --force-new    Fails if output file already exists
    -h, --help         Prints help information
    -p, --pretty       Prettify json output
    -V, --version      Prints version information

OPTIONS:
    -o, --output <filename>    Writes the output into a file

ARGS:
    <query>    Filter and pipeline query

EXAMPLES :

	sjq "." : Ouptut everything (useful with --pretty option to prettify the input)

	sjq ".field_name" : For each object in the object stream input (maybe just one), output the content of the field "field_name"

	sjq '."field_name with space"' : Same as previous, but for a field containing spaces or other special characters

	sjq "./field_(name|value)(_\d+)?/" : Same as previous, but using regex. Here this query matches the fields "field_name", "field_value", "field_name_192", ...
```