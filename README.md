<div align="center">

# ✒️ lazycard

A simple flashcard application for the terminal.

TODO: image

</div>

## 📌 Features

- Write cards in a light markup
- Card scheduling using [FSRS](https://github.com/open-spaced-repetition/fsrs4anki/wiki/The-Algorithm)

## ⚡ Usage

The `lazycard` command takes no mandatory arguments, but you can supply it with options for where your database file should be, along with the assets directory and the settings file.

```console
Usage: lazycard [OPTIONS]

Example: lazycard --database /path/to/my/database.db --assets /path/to/my/assets

Options:
      --database <FILE.db>    Optional path for your database file. By default, the location will be determined by the conventions of your operating system
      --assets <DIR>          Optional path for your assets directory. By default, the location will be determined by the conventions of your operating system
      --settings <FILE.toml>  Optional path for your settings file. By default, the location will be determined by the conventions of your operating system
  -h, --help                  Print help
  -V, --version               Print version
```

## 📜 Markup

Cards are written in a custom lightweight markup language, inspired by both [Markdown](https://en.wikipedia.org/wiki/Markdown) and [Djot](https://djot.net/). It provides a small set of syntax for formatting text in the terminal. The following table shows the entire syntax available.

<table align="center">
<tr>
<th>Markup</th>
<th>Result</th>
</tr>
<tr>
<td>

<pre>
A normal paragraph with *bold* and _italic_ text.

| Center paragraph

> Right paragraph

---

- item 1
- item 2

# This is a comment
![image description](image.jpeg)

```python
def add(a, b):
    return a + b
```
</pre>

</td>
<td>

<p>A normal paragraph with <b>bold</b> and <i>italic</i> text.</p>
<p align="center">Center paragraph</p>
<p align="right">Right paragraph</p>

<hr>

- item 1
- item 2

<div align="center">
<figure>
<img src="https://github.com/user-attachments/assets/269f45c2-3164-4310-a7c3-0ad898e951e7"/>
<p>image description</p>
</figure>
</div>

<pre>
def add(a, b):
    return a + b
</pre>

</td>
</tr>
</table>

## 🔖 Install

todo
