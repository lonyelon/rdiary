# RDiary

Simple diary writer for Linux.

This is a very personal app, as it is what I use to write it. I will add features as I need them.

## How to use

You need to export the `DIARY_DIR` environment variable to specify the directory where the files will go, for example:

```sh
export DIARY_DIR=~/Documents/diary
```

Once that is done, the program can be run without problems.

Note: the editor for entries is taken from the `EDITOR` environment variable.
If not set, RDiary will default to `vim`.
