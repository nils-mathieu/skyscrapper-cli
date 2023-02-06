# Information

A simple CLI application that allows users to generate valid Skyscrapper boards.

## Installation

### Release Page

If you are on linux x86_64, you can download the latest version of this tool from the [releases](https://github.com/nils-mathieu/skyscrapper-cli/releases) section on github.

### Building From Source

If you have Rust installed on your machine, you can use the following commands to download and compile it into a binary file.

```txt
git clone https://github.com/nils-mathieu/skyscrapper-cli
cd skyscrapper-cli
cargo build --release
```

The result of that command will usually be located in `target/release/skyscrapper-cli`, but that might change if you have defined a custom target directory for Cargo.

## Examples

Basic usage:

```txt
>_ ./skyscrapper-cli generate 4
  3 1 2 3
2 2 4 3 1 3
3 1 3 4 2 2
2 3 1 2 4 1
1 4 2 1 3 2
  1 3 3 2
```

It's possible to require a specific output format.

```txt
>_ ./skyscrapper-cli generate -o solution 4
4 1 3 2
3 2 4 1
1 3 2 4
2 4 1 3

>_ ./skyscrapper-cli generate -o header 4
  1 4 2 2
1         3
2         2
3         1
2         2
  3 1 3 2

>_ ./skyscrapper-cli generate -o both 4
  1 4 2 2
1 4 1 3 2 3
2 3 2 4 1 2
3 1 3 2 4 1
2 2 4 1 3 2
  3 1 3 2

>_ ./skyscrapper-cli generate -o header-line 4
1 4 2 2 3 1 3 2 1 2 3 2 3 2 1 2
```

You can even require multiple output formats at once.

```txt
>_ ./skyscrapper-cli generate -o header-line -o solution 4
1 4 2 2 3 1 3 2 1 2 3 2 3 2 1 2

4 1 3 2
3 2 4 1
1 3 2 4
2 4 1 3
```

If you want reproductible results, you can use the `--seed` option. Using twice the same seed will result in twice the same board.

```txt
>_ ./skyscrapper-cli generate --seed 12312323 5
  3 2 3 1 2
2 3 1 2 5 4 2
2 4 5 1 3 2 3
3 1 4 3 2 5 1
3 2 3 5 4 1 3
1 5 2 4 1 3 3
  1 4 2 3 2

>_ ./skyscrapper-cli generate --seed 12312323 5
  3 2 3 1 2
2 3 1 2 5 4 2
2 4 5 1 3 2 3
3 1 4 3 2 5 1
3 2 3 5 4 1 3
1 5 2 4 1 3 3
  1 4 2 3 2
```

It's possible to solve the skyscrapper problem using a given header-line.

```txt
>_ ./skyscrapper-cli solve "1 4 2 2 3 1 3 2 1 2 3 2 3 2 1 2"
  1 4 2 2
1 4 1 3 2 3
2 3 2 4 1 2
3 1 3 2 4 1
2 2 4 1 3 2
  3 1 3 2
```

Or check whether a given solution is valid or not.

```txt
>_ << EOF ./skyscrapper-cli check "1 4 2 2 3 1 3 2 1 2 3 2 3 2 1 2"
4 1 3 2
3 2 4 1
1 3 2 4
2 4 1 3
EOF
>_ echo $?
0
```
