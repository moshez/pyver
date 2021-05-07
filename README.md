# pyver

A Python version manager.

## build     

```
pver build --no-dry-run --version <Python version>
````

will build the given CPython version and make it available.

## which

```
pyver which <Partial version>
```

will print out a path to a Python binary that matches the version.

For example

```
$ pyver which 3
/home/moshez/.pyver/versions/3.9.2/bin/python3
```

You can use it to create virtual environments:

```
$ $(pyver which 3) -m venv ~/.venvs/my-special-env
``` 

## Default directory


PyVer defaults to storing Python builds in
`$PYVER_ROOT`,
if defined,
or `~/.pyver`,
if not.
