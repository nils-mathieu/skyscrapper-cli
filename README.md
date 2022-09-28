# Information

A simple CLI application that allows users to generate valid Skyscrapper boards.

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

If you want reproductible results, you can use the `--seed` option. Using twice the same seed will result in twice the same board.

```txt
>_ ./skyscrapper-cli generate --seed 12314344 5
  4 2 1 2
3 1 2 4 3 2
3 2 1 3 4 1
2 3 4 2 1 3
1 4 3 1 2 3
  1 2 4 2

>_ ./skyscrapper-cli generate --seed 12314344 5
  4 2 1 2
3 1 2 4 3 2
3 2 1 3 4 1
2 3 4 2 1 3
1 4 3 1 2 3
  1 2 4 2
```
